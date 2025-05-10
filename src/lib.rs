mod backends;
mod keys;
mod kv_error;
mod kv_value;
mod list_builder;

pub use crate::backends::{KvBackend, memory_backend::MemoryBackend};
pub use crate::keys::{KvKey, display};
pub use crate::kv_error::{KvError, KvResult};
pub use crate::kv_value::KvValue;
pub use crate::list_builder::KvListBuilder;
pub use keys::IntoKey;
use keys::display::{parse_display_string_to_key, to_display_string};

#[cfg(feature = "sqlite")]
pub use crate::backends::sqlite_backend::SqliteBackend;

pub struct Kv<'a> {
    backend: &'a mut dyn KvBackend,
}

impl<'a> Kv<'a> {
    pub fn new(backend: &'a mut dyn KvBackend) -> Self {
        Self { backend }
    }

    pub fn get(&self, key: &dyn IntoKey) -> KvResult<Option<KvValue>> {
        let key = key.to_key();
        let pairs = self.backend.get_range(Some(key.clone()), Some(key))?;
        if pairs.is_empty() {
            Ok(None)
        } else {
            let (decoded, _) =
                bincode::decode_from_slice::<KvValue, _>(&pairs[0].1, bincode::config::standard())
                    .map_err(KvError::ValDecodeError)?;
            Ok(Some(decoded))
        }
    }

    pub fn set(&mut self, key: &dyn IntoKey, value: KvValue) -> KvResult<()> {
        self.set_optional(key, Some(value))
    }

    pub(crate) fn set_optional(
        &mut self,
        key: &dyn IntoKey,
        value: Option<KvValue>,
    ) -> KvResult<()> {
        let key = key.to_key();
        if let Some(v) = value {
            let encoded = bincode::encode_to_vec(v, bincode::config::standard())
                .map_err(KvError::ValEncodeError)?;
            self.backend.set(key, Some(encoded))
        } else {
            // Remove the key completely!
            self.backend.set(key, None)
        }
    }

    pub fn delete(&mut self, key: &dyn IntoKey) -> KvResult<Option<(KvKey, KvValue)>> {
        let made = key.to_key();
        let val = self.get(key)?;
        if let Some(val) = val {
            self.set_optional(key, None)?;
            Ok(Some((made, val)))
        } else {
            Ok(None)
        }
    }

    pub fn entries(&'a mut self) -> KvResult<Vec<(KvKey, KvValue)>> {
        KvListBuilder {
            backend: self.backend,
            start: None,
            end: None,
            prefix: None,
        }
        .entries()
    }

    pub fn list(&'a mut self) -> KvListBuilder<'a> {
        KvListBuilder::new(self.backend)
    }

    pub fn to_serde_json(&'a mut self) -> KvResult<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (key, value) in self.entries()? {
            let display = to_display_string(&key.0).ok_or(KvError::KeyDecodeError(format!(
                "Invalid key {key:#?}.\nThis should never happen, please file a bug report."
            )))?;
            map.insert(display, serde_json::Value::from(&value));
        }
        Ok(serde_json::Value::Object(map))
    }

    pub fn from_serde_json(
        backend: &'a mut dyn KvBackend,
        json: serde_json::Value,
    ) -> KvResult<Self> {
        if let Some(obj) = json.as_object() {
            let mut kv = Self::new(backend);
            for (display, value) in obj.iter() {
                let key = parse_display_string_to_key(display).ok_or(KvError::KeyDecodeError(
                    format!("Could not decode JSON key {display} to KvKey."),
                ))?;
                kv.set(&key, KvValue::from(value))?;
            }
            Ok(kv)
        } else {
            Err(KvError::Other(format!(
                "Invalid JSON value while trying to make Kv from serde_json::Value: {json}"
            )))
        }
    }

    pub fn dump_json(&'a mut self) -> KvResult<String> {
        let json = self.to_serde_json()?;
        Ok(json.to_string())
    }

    pub fn from_json_string(backend: &'a mut dyn KvBackend, json: String) -> KvResult<Self> {
        let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&json)
            .map_err(|e| KvError::Other(format!("serde error parsing json: {e}")))?;
        Self::from_serde_json(backend, serde_json::Value::Object(json))
    }
}

#[cfg(test)]
mod kv_integration_tests {
    use super::*;
    use crate::{Kv, KvValue, keys::IntoKey};

    #[test]
    fn set_and_get_single_value() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);

        let tup = (1001u64, "foo".to_string());
        let key = tup.to_key();
        let value = KvValue::String("bar".to_string());

        kv.set(&tup, value.clone())?;
        let out = kv.get(&key)?;

        assert_eq!(out, Some(value));
        Ok(())
    }

    #[test]
    fn get_nonexistent_returns_none() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let kv = Kv::new(&mut backend);
        let tup = (9u64, "not_there".to_string());
        let out = kv.get(&tup)?;
        assert_eq!(out, None);
        Ok(())
    }

    #[test]
    fn delete_removes_value() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);

        let tup = (512u64, "z".to_string());
        let key = tup.to_key();
        kv.set(&tup, KvValue::Bool(true))?;
        let del = kv.delete(&key)?;
        assert!(del.is_some());
        let after = kv.get(&key)?;
        assert!(after.is_none());
        Ok(())
    }

    #[test]
    fn overwrite_value() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);

        let tup = (17u64, "a".to_string());
        kv.set(&tup, KvValue::U64(123))?;
        kv.set(&tup, KvValue::U64(456))?;

        let out = kv.get(&tup)?;
        assert_eq!(out, Some(KvValue::U64(456)));
        Ok(())
    }

    #[test]
    fn list_prefix() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);

        for i in 0..10u64 {
            let tup = (1u64, i);
            kv.set(&tup, KvValue::U64(i))?;
        }
        for j in 0..10u64 {
            let tup = (2u64, j);
            kv.set(&tup, KvValue::U64(j))?;
        }
        // List all keys starting with (1, _)
        let results = kv.list().prefix(&(1u64,)).entries()?;
        assert_eq!(results.len(), 10);
        for (k, v) in results {
            let (_prefix, idx): (u64, u64) = k.try_into()?;
            assert_eq!(v, KvValue::U64(idx));
        }
        Ok(())
    }

    #[test]
    fn list_range() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);

        for i in 1..=5u64 {
            let tup = (99u64, i);
            kv.set(&tup, KvValue::U64(i * 10))?;
        }
        // List from (99,2) up to but not including (99,5)
        let results = kv
            .list()
            .start(&(99u64, 2u64))
            .end(&(99u64, 5u64))
            .entries()?;

        let want = vec![20, 30, 40];
        let got: Vec<u64> = results
            .into_iter()
            .map(|(_k, v)| if let KvValue::U64(n) = v { n } else { 0 })
            .collect();

        assert_eq!(got, want);
        Ok(())
    }

    #[test]
    fn clear_backend() -> KvResult<()> {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);

        for i in 0..5u64 {
            let tup = (777u64, i);
            kv.set(&tup, KvValue::U64(i))?;
        }
        kv.backend.clear()?;
        let items = kv.entries()?;
        assert_eq!(items.len(), 0);
        Ok(())
    }
}
