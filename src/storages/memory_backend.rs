// src/storages/MemoryBackend.rs
use crate::storages::kv_backend::{KvBackend, KvResult};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct MemoryBackend {
    map: HashMap<Vec<u8>, Vec<u8>>,
}
impl MemoryBackend {
    pub fn new() -> Self {
        Self::default()
    }
}
impl KvBackend for MemoryBackend {
    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> KvResult<()> {
        self.map.insert(key, value);
        Ok(())
    }
    fn get(&self, key: &[u8]) -> KvResult<Option<Vec<u8>>> {
        Ok(self.map.get(key).cloned())
    }
    fn delete(&mut self, key: &[u8]) -> KvResult<()> {
        self.map.remove(key);
        Ok(())
    }
    fn clear(&mut self) -> KvResult<()> {
        self.map.clear();
        Ok(())
    }
    fn get_many<'a>(
        &'a self,
        keys: Vec<Vec<u8>>,
    ) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + 'a>> {
        let keyset: HashSet<Vec<u8>> = keys.into_iter().collect();
        Ok(Box::new(
            self.map
                .iter()
                .filter(move |(k, _)| keyset.contains(k.as_slice()))
                .map(|(_, v)| v.clone()),
        ))
    }
    fn keys<'a>(&'a self) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + 'a>> {
        Ok(Box::new(self.map.keys().cloned()))
    }
}
