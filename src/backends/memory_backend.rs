use std::collections::BTreeMap;
use std::ops::Bound;
use std::sync::{Arc, Mutex};

use crate::{KvBackend, KvKey, KvResult};

#[derive(Debug, Default, Clone)]
pub struct MemoryBackend {
    // Shared and thread-safe
    map: Arc<Mutex<BTreeMap<KvKey, Vec<u8>>>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl KvBackend for MemoryBackend {
    fn get_range(
        &self,
        start: Option<KvKey>,
        end: Option<KvKey>,
    ) -> KvResult<Vec<(KvKey, Vec<u8>)>> {
        let map = self.map.lock().unwrap();

        let range = match (start, end) {
            (Some(start_key), Some(end_key)) => {
                if start_key == end_key {
                    map.range((Bound::Included(start_key), Bound::Included(end_key)))
                } else {
                    map.range(start_key..end_key)
                }
            }
            (Some(start_key), None) => map.range(start_key..),
            (None, Some(end_key)) => map.range(..end_key),
            (None, None) => map.range::<KvKey, _>(..),
        };

        Ok(range.map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    fn set(&mut self, key: KvKey, value: Option<Vec<u8>>) -> KvResult<()> {
        let mut map = self.map.lock().unwrap();
        if let Some(v) = value {
            map.insert(key, v);
        } else {
            map.remove(&key);
        }
        Ok(())
    }

    fn clear(&mut self) -> KvResult<()> {
        let mut map = self.map.lock().unwrap();
        map.clear();
        Ok(())
    }
}
