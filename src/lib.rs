pub mod keys;

pub mod storages {
    pub mod kv_backend;
    pub mod memory_backend;
    #[cfg(feature = "sqlite")]
    pub mod sqlite_backend;
}

pub mod utils {
    pub mod list_builder;
    pub use list_builder::KvListBuilder;
}

pub use keys::{Key, DecodeError, FromKey, IntoKey};
pub use storages::kv_backend::{KvBackend, KvResult};
pub use storages::memory_backend::MemoryBackend;
pub use utils::KvListBuilder;

pub struct Kv<B: KvBackend> {
    backend: B,
}

impl<B: KvBackend> Kv<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    pub fn set<K, T>(&mut self, key: K, value: T) -> KvResult<()>
    where
        K: IntoKey,
        T: bincode::Encode,
    {
        let key = key.into_key();
        let bytes = bincode::encode_to_vec(value, bincode::config::standard())?;
        self.backend.set(key, bytes)
    }
    pub fn get<K, T>(&self, key: K) -> KvResult<Option<T>>
    where
        K: IntoKey,
        T: bincode::Decode<()>,
    {
        let key = key.into_key();
        match self.backend.get(&key)? {
            Some(bytes) => Ok(
                bincode::decode_from_slice(&bytes, bincode::config::standard())
                    .ok()
                    .map(|(value, _)| value),
            ),
            None => Ok(None),
        }
    }
    pub fn delete<K>(&mut self, key: K) -> KvResult<()>
    where
        K: IntoKey,
    {
        let key = key.into_key();
        self.backend.delete(&key)
    }
    pub fn clear(&mut self) -> KvResult<()> {
        self.backend.clear()
    }
    pub fn keys(&self) -> KvResult<impl Iterator<Item = Key> + '_> {
        self.backend.keys()
    }
    pub fn list<T>(&self) -> KvListBuilder<B, T> {
        KvListBuilder::new(&self.backend)
    }
}

#[cfg(test)]
mod tests;
