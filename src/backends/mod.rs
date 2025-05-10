use crate::{KvKey, KvResult};

pub(crate) mod memory_backend;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite_backend;

pub trait KvBackend {
    fn get_range(
        &self,
        start: Option<KvKey>,
        end: Option<KvKey>,
    ) -> KvResult<Vec<(KvKey, Vec<u8>)>>;
    fn set(&mut self, key: KvKey, value: Option<Vec<u8>>) -> KvResult<()>;
    fn clear(&mut self) -> KvResult<()>;
}
