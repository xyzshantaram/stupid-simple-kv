use bincode::config::{BigEndian, Configuration, NoLimit, Varint};
use std::{collections::HashMap, marker::PhantomData};

/// A key-value store abstraction over an underlying backend.
pub struct Kv<'a> {
    storage: &'a mut dyn KvBackend,
}

const BINCODE_CONFIG: Configuration<BigEndian, Varint, NoLimit> = bincode::config::standard()
    .with_big_endian()
    .with_variable_int_encoding();

impl<'a> Kv<'a> {
    /// Creates a new key-value store with the given backend.
    pub fn new(storage: &'a mut dyn KvBackend) -> Self {
        Kv { storage }
    }

    /// Stores a serializable value by key.
    pub fn set<T: bincode::Encode, K: Into<String>>(
        &mut self,
        key: K,
        value: T,
    ) -> Result<(), anyhow::Error> {
        let key = key.into();
        let bytes = bincode::encode_to_vec(value, BINCODE_CONFIG)?;
        self.storage.set(key, bytes);
        Ok(())
    }

    /// Retrieves a deserializable value by key.
    pub fn get<T: bincode::Decode<()>, K: Into<String>>(&self, key: K) -> Option<T> {
        let key = key.into();
        self.storage.get(key).and_then(|bytes| {
            bincode::decode_from_slice(&bytes, BINCODE_CONFIG)
                .ok()
                .map(|(value, _)| value)
        })
    }

    /// Deletes the value for a key.
    pub fn delete<K: Into<String>>(&mut self, key: K) {
        let key = key.into();
        self.storage.delete(key);
    }

    /// Removes all keys and values.
    pub fn clear(&mut self) {
        self.storage.clear();
    }

    /// Begins building a filtered value iterator.
    pub fn list<'b, T: bincode::Decode<()>>(&'b self) -> ListBuilder<'b, 'a, T> {
        ListBuilder::new(self)
    }

    /// Returns an iterator over all keys.
    pub fn keys_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        self.storage.keys_iter()
    }
}

/// Trait representing a pluggable key-value backend.
pub trait KvBackend {
    /// Stores a value by key.
    fn set(&mut self, key: String, value: Vec<u8>);
    /// Gets a value by key, if present.
    fn get(&self, key: String) -> Option<Vec<u8>>;
    /// Deletes the value for a key.
    fn delete(&mut self, key: String);
    /// Deletes all keys and values.
    fn clear(&mut self);
    /// Returns an iterator over values for supplied keys.
    fn get_many_iter<'a>(&'a self, keys: Vec<String>) -> Box<dyn Iterator<Item = Vec<u8>> + 'a>;
    /// Returns an iterator over all keys.
    fn keys_iter<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a>;
}

/// Helper for building filtered value iterators from a Kv store.
pub struct ListBuilder<'a, 'b, T> {
    store: &'a Kv<'b>,
    start: Option<String>,
    end: Option<String>,
    prefix: Option<String>,
    _marker: PhantomData<T>,
}

impl<'a, 'b, T: bincode::Decode<()>> ListBuilder<'a, 'b, T> {
    /// Creates a new builder for listing values.
    pub fn new(store: &'a Kv<'b>) -> Self {
        Self {
            store,
            start: None,
            end: None,
            prefix: None,
            _marker: PhantomData,
        }
    }

    /// Filter results to keys starting from this (inclusive).
    pub fn start<S: Into<String>>(mut self, key: S) -> Self {
        self.start = Some(key.into());
        self
    }

    /// Filter results to keys ending at this (inclusive).
    pub fn end<S: Into<String>>(mut self, key: S) -> Self {
        self.end = Some(key.into());
        self
    }

    /// Filter results to keys with this prefix.
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Returns an iterator over the filtered, deserialized values.
    pub fn iter(self) -> Result<Box<dyn Iterator<Item = T> + 'a>, &'static str> {
        let num_supplied =
            self.start.is_some() as u32 + self.end.is_some() as u32 + self.prefix.is_some() as u32;

        if num_supplied == 0 || num_supplied == 3 {
            return Err(
                "You must set exactly one or two of `start`, `end`, and `prefix`, but not all three.",
            );
        }

        let keys_iter = self.store.keys_iter();
        let filtered_keys: Vec<String> = match (self.start, self.end, self.prefix) {
            // prefix only
            (None, None, Some(ref prefix)) => {
                keys_iter.filter(move |k| k.starts_with(prefix)).collect()
            }
            // start only
            (Some(ref start), None, None) => {
                let mut passed = false;
                keys_iter
                    .filter(|k| {
                        if !passed && k == start {
                            passed = true;
                            true
                        } else {
                            passed
                        }
                    })
                    .collect()
            }
            // end only
            (None, Some(ref end), None) => keys_iter.take_while(|k| k <= end).collect(),
            // start + end
            (Some(ref start), Some(ref end), None) => {
                let mut passed = false;
                keys_iter
                    .filter(|k| {
                        if !passed && k == start {
                            passed = true;
                            true
                        } else {
                            passed
                        }
                    })
                    .take_while(|k| k <= end)
                    .collect()
            }
            // prefix + start
            (Some(ref start), None, Some(ref prefix)) => {
                let mut passed = false;
                keys_iter
                    .filter(move |k| k.starts_with(prefix))
                    .filter(|k| {
                        if !passed && k == start {
                            passed = true;
                            true
                        } else {
                            passed
                        }
                    })
                    .collect()
            }
            // prefix + end
            (None, Some(ref end), Some(ref prefix)) => keys_iter
                .filter(move |k| k.starts_with(prefix))
                .take_while(|k| k <= end)
                .collect(),
            _ => return Err("Invalid combination of start, end, and prefix"),
        };

        Ok(Box::new(
            self.store
                .storage
                .get_many_iter(filtered_keys)
                .filter_map(|bytes| {
                    bincode::decode_from_slice(&bytes, BINCODE_CONFIG)
                        .ok()
                        .map(|(v, _)| v)
                }),
        ))
    }
}

/// In-memory implementation of KvBackend using a HashMap.
#[derive(Default)]
pub struct MemoryBackend {
    map: HashMap<String, Vec<u8>>,
}

impl MemoryBackend {
    /// Creates a new in-memory backend.
    pub fn new() -> Self {
        Self::default()
    }
}

impl KvBackend for MemoryBackend {
    fn set(&mut self, key: String, value: Vec<u8>) {
        self.map.insert(key, value);
    }

    fn get(&self, key: String) -> Option<Vec<u8>> {
        self.map.get(&key).cloned()
    }

    fn delete(&mut self, key: String) {
        self.map.remove(&key);
    }

    fn clear(&mut self) {
        self.map.clear();
    }

    fn get_many_iter<'a>(&'a self, keys: Vec<String>) -> Box<dyn Iterator<Item = Vec<u8>> + 'a> {
        Box::new(
            self.map
                .iter()
                .filter(move |(key, _)| keys.contains(key))
                .map(|item| item.1.clone()),
        )
    }

    fn keys_iter<'a>(&'a self) -> Box<dyn Iterator<Item = String> + 'a> {
        Box::new(self.map.keys().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_backend_keys_are_owned() {
        let mut backend = MemoryBackend::new();
        backend.set("foo".into(), vec![1, 2, 3]);
        assert_eq!(backend.get("foo".into()), Some(vec![1, 2, 3]));

        backend.delete("foo".into());
        assert_eq!(backend.get("foo".into()), None);
    }

    #[test]
    fn kv_interface_accepts_str_and_string() {
        // Accepts str and String types for keys
    }

    #[test]
    fn get_and_delete_nonexistent_key() {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        assert_eq!(kv.get::<u32, _>("nope"), None);
        kv.delete("nope"); // Should not panic
    }

    #[test]
    fn clear_removes_all_keys() {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        kv.set("a", 123i32).unwrap();
        kv.set("b", 456i32).unwrap();
        kv.clear();
        assert_eq!(kv.get::<i32, _>("a"), None);
        assert_eq!(kv.get::<i32, _>("b"), None);
        assert_eq!(kv.keys_iter().count(), 0);
    }

    #[test]
    fn keys_iterator_matches_inserts_and_deletes() {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        kv.set("a", 1).unwrap();
        kv.set("b", 2).unwrap();
        kv.set("c", 3).unwrap();
        let mut keys: Vec<_> = kv.keys_iter().collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);
        kv.delete("b");
        let mut keys: Vec<_> = kv.keys_iter().collect();
        keys.sort();
        assert_eq!(keys, vec!["a", "c"]);
    }

    #[test]
    fn can_list_values_with_listbuilder() {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        kv.set("k_a", 10u8).unwrap();
        kv.set("k_b", 20u8).unwrap();
        kv.set("k_c", 30u8).unwrap();
        let vals: Vec<u8> = kv.list().prefix("k_").iter().unwrap().collect();
        assert!(vals.contains(&10));
        assert!(vals.contains(&20));
        assert!(vals.contains(&30));
        assert_eq!(vals.len(), 3);
    }

    #[test]
    fn can_store_mixed_types() {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        kv.set("i", 123i64).unwrap();
        kv.set("s", "hello".to_string()).unwrap();
        assert_eq!(kv.get::<i64, _>("i"), Some(123));
        assert_eq!(kv.get::<String, _>("s"), Some("hello".to_string()));
    }

    #[test]
    fn kv_interface_accepts_str_and_string_original() {
        let mut backend = MemoryBackend::new();
        let mut kv = Kv::new(&mut backend);
        kv.set("k1", 42u32).unwrap();
        kv.set("k2".to_string(), 54u32).unwrap();
        assert_eq!(kv.get::<u32, _>("k1"), Some(42));
        assert_eq!(kv.get::<u32, _>("k2"), Some(54));
    }
}
