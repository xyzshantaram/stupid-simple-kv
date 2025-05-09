//! Crate root for stupid-simple-kv: a pluggable, minimal key-value store library.

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
    pub mod kv_value;
    pub use kv_value::KvValue;
}

pub use keys::{DecodeError, FromKey, IntoKey, Key};
pub use storages::kv_backend::{KvBackend, KvResult};
pub use storages::memory_backend::MemoryBackend;
pub use utils::KvListBuilder;

/// The main `Kv` type providing simple get/set/delete/clear/key/list operations on a pluggable backend.
pub struct Kv<B: KvBackend> {
    backend: B,
}

impl<B: KvBackend> Kv<B> {
    /// Create a new key-value store instance with the provided backend.
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    /// Set a value for a key.
    pub fn set<K, T>(&mut self, key: K, value: T) -> KvResult<()>
    where
        K: IntoKey,
        T: serde::Serialize + 'static,
    {
        use crate::utils::KvValue;
        let key = key.into_key();
        let kvv = KvValue::from_any(&value)?;
        let bytes = bincode::encode_to_vec(kvv, bincode::config::standard())?;
        self.backend.set(key, bytes)
    }
    /// Retrieve a value for a key.
    pub fn get<K, T>(&self, key: K) -> KvResult<Option<T>>
    where
        K: IntoKey,
        T: serde::de::DeserializeOwned + 'static,
    {
        use crate::utils::KvValue;
        let key = key.into_key();
        match self.backend.get(&key)? {
            Some(bytes) => {
                let (val, _): (KvValue, _) =
                    bincode::decode_from_slice(&bytes, bincode::config::standard())?;
                Ok(val.to_any()?)
            }
            None => Ok(None),
        }
    }
    /// Delete a value for a key.
    pub fn delete<K>(&mut self, key: K) -> KvResult<()>
    where
        K: IntoKey,
    {
        let key = key.into_key();
        self.backend.delete(&key)
    }
    /// Remove all data.
    pub fn clear(&mut self) -> KvResult<()> {
        self.backend.clear()
    }
    /// Return an iterator over all keys.
    pub fn keys(&self) -> KvResult<impl Iterator<Item = Key> + '_> {
        self.backend.keys()
    }
    /// Create a list builder for filtered/ordered iteration.
    pub fn list<T>(&self) -> KvListBuilder<B, T> {
        KvListBuilder::new(&self.backend)
    }
}

#[cfg(test)]
mod tests;
