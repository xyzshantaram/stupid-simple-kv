// src/storages/KvBackend.rs
pub type KvError = Box<dyn std::error::Error + Send + Sync>;
pub type KvResult<T> = Result<T, KvError>;

/// A pluggable key-value backend trait.
pub trait KvBackend {
    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> KvResult<()>;
    fn get(&self, key: &[u8]) -> KvResult<Option<Vec<u8>>>;
    fn delete(&mut self, key: &[u8]) -> KvResult<()>;
    fn clear(&mut self) -> KvResult<()>;
    fn get_many<'a>(&'a self, keys: Vec<Vec<u8>>) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + 'a>>;
    fn keys<'a>(&'a self) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + 'a>>;
}
