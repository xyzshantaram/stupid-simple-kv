/// In-memory key-value backend using `HashMap`.
use crate::keys::Key;
use crate::storages::kv_backend::{KvBackend, KvResult};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct MemoryBackend {
    map: HashMap<Key, Vec<u8>>,
}
impl MemoryBackend {
    pub fn new() -> Self {
        Self::default()
    }
}
impl KvBackend for MemoryBackend {
    fn set(&mut self, key: Key, value: Vec<u8>) -> KvResult<()> {
        self.map.insert(key, value);
        Ok(())
    }
    fn get(&self, key: &Key) -> KvResult<Option<Vec<u8>>> {
        Ok(self.map.get(key).cloned())
    }
    fn delete(&mut self, key: &Key) -> KvResult<()> {
        self.map.remove(key);
        Ok(())
    }
    fn clear(&mut self) -> KvResult<()> {
        self.map.clear();
        Ok(())
    }
    fn get_many(
        &self,
        keys: Vec<Key>,
    ) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + Send + Sync + 'static>> {
        let keyset: HashSet<Key> = keys.into_iter().collect();
        let results = self
            .map
            .iter()
            .filter(move |(k, _)| keyset.contains(k))
            .map(|(_, v)| v.clone())
            .collect::<Vec<_>>()
            .into_iter();
        Ok(Box::new(results))
    }
    fn keys(&self) -> KvResult<Box<dyn Iterator<Item = Key> + Send + Sync + 'static>> {
        let keys = self.map.keys().cloned().collect::<Vec<_>>().into_iter();
        Ok(Box::new(keys))
    }
}
