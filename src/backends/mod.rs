use crate::{KvKey, KvResult};

pub(crate) mod memory_backend;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite_backend;

/// Trait for all key-value store backends.
///
/// Backends must provide the following semantics:
/// - **Keys are encoded, ordered byte strings**: All key operations should respect the lexicographic ordering of the encoded bytes, as provided by [`KvKey`].
/// - **Atomicity**: `set` and `clear` must complete their operation or return an error.
/// - **Value format**: Values must be raw binary blobs. Serialization and deserialization are handled by the library; the backend just stores the [`u8`] arrays.
/// - **Iteration**: `get_range` should return all keys in `[start, end)` order. If `end` is `None`, iteration should go until the end of the keyspace.
/// - **Error Reporting**: All failures must return a [`KvResult::Err`] with a suitable error value.
///
/// See [`memory_backend`] and (if enabled) [`sqlite_backend`] for correct implementation templates.
pub trait KvBackend {
    fn get_range(
        &self,
        start: Option<KvKey>,
        end: Option<KvKey>,
    ) -> KvResult<Vec<(KvKey, Vec<u8>)>>;
    fn set(&mut self, key: KvKey, value: Option<Vec<u8>>) -> KvResult<()>;
    fn clear(&mut self) -> KvResult<()>;
}
