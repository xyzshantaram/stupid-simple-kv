mod keys;
pub use keys::KeyEncoder;

pub mod storages {
    pub mod kv_backend;
    pub mod memory_backend;
    #[cfg(feature = "sqlite")]
    pub mod sqlite_backend;
}

pub use storages::kv_backend::{KvBackend, KvResult};
pub use storages::memory_backend::MemoryBackend;
pub mod utils {
    pub mod list_builder;
    pub use list_builder::KvListBuilder;
}
pub use utils::KvListBuilder;

pub struct Kv<B: KvBackend> {
    backend: B,
}

impl<B: KvBackend> Kv<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    pub fn set<T: bincode::Encode>(&mut self, key: Vec<u8>, value: T) -> KvResult<()> {
        let bytes = bincode::encode_to_vec(value, bincode::config::standard())?;
        self.backend.set(key, bytes)
    }
    pub fn get<T: bincode::Decode<()>>(&self, key: &[u8]) -> KvResult<Option<T>> {
        match self.backend.get(key)? {
            Some(bytes) => Ok(
                bincode::decode_from_slice(&bytes, bincode::config::standard())
                    .ok()
                    .map(|(value, _)| value),
            ),
            None => Ok(None),
        }
    }
    pub fn delete(&mut self, key: &[u8]) -> KvResult<()> {
        self.backend.delete(key)
    }
    pub fn clear(&mut self) -> KvResult<()> {
        self.backend.clear()
    }
    pub fn keys(&self) -> KvResult<impl Iterator<Item = Vec<u8>> + '_> {
        self.backend.keys()
    }
    pub fn list<T>(&self) -> KvListBuilder<B, T> {
        KvListBuilder::new(&self.backend)
    }
}

#[cfg(test)]
mod tests;
