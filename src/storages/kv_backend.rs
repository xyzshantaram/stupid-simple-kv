use crate::keys::Key;
pub type KvError = Box<dyn std::error::Error + Send + Sync>;
pub type KvResult<T> = Result<T, KvError>;

/// A pluggable key-value backend trait.
pub trait KvBackend {
    fn set(&mut self, key: Key, value: Vec<u8>) -> KvResult<()>;
    fn get(&self, key: &Key) -> KvResult<Option<Vec<u8>>>;
    fn delete(&mut self, key: &Key) -> KvResult<()>;
    fn clear(&mut self) -> KvResult<()>;
    fn get_many(&self, keys: Vec<Key>) -> KvResult<Box<dyn Iterator<Item = Vec<u8>> + Send + Sync + 'static>>;
    fn keys(&self) -> KvResult<Box<dyn Iterator<Item = Key> + Send + Sync + 'static>>;
}
