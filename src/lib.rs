mod backends;
mod keys;
mod kv_error;
mod kv_value;
mod list_builder;
mod tests;

use std::marker::PhantomData;

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
    backend: Box<dyn KvBackend>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Kv<'a> {
    pub fn new(backend: Box<dyn KvBackend>) -> Self {
        Self {
            backend,
            _marker: PhantomData,
        }
    }

    pub fn get(&self, key: &dyn IntoKey) -> KvResult<Option<KvValue>> {
        let key = key.to_key();
        let pairs = self.backend.get_range(Some(key.clone()), key.successor())?;
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

    pub fn entries(&mut self) -> KvResult<Vec<(KvKey, KvValue)>> {
        KvListBuilder {
            backend: &mut self.backend,
            start: None,
            end: None,
            prefix: None,
        }
        .entries()
    }

    pub fn list(&'a mut self) -> KvListBuilder<'a> {
        KvListBuilder::new(&mut self.backend)
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

    pub fn from_serde_json(backend: Box<dyn KvBackend>, json: serde_json::Value) -> KvResult<Self> {
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

    pub fn from_json_string(backend: Box<dyn KvBackend>, json: String) -> KvResult<Self> {
        let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&json)
            .map_err(|e| KvError::Other(format!("serde error parsing json: {e}")))?;
        Self::from_serde_json(backend, serde_json::Value::Object(json))
    }
}
