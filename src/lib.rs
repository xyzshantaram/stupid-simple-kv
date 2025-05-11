//! # stupid-simple-kv
//!
//! A dead-simple, pluggable, and binary-sorted key-value store for Rust.
//!
//! ## Features
//!
//! - **Order-preserving tuple-style keys**: Compose keys using `u64`, `i64`, `bool`, `String`, tuples, or your own struct if it implements [`IntoKey`].
//! - **Pluggable design**: Swap between memory or SQLite backends, or define your own by implementing [`KvBackend`].
//! - **Automatic value serialization**: Store any serde-serializable value as a [`KvValue`].
//! - **List/query API**: Filter or range-scan with [`KvListBuilder`].
//! - **Easy JSON import/export**: Dump or restore the store's contents for debugging or migration.
//! - **Typed errors** and strict Rust interface.
//!
//! ## Quickstart
//!
//! ```rust
//! use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
//!
//! let backend = Box::new(MemoryBackend::new());
//! let mut kv = Kv::new(backend);
//!
//! let key = (42u64, true, -17i64, "foo").to_key();
//! kv.set(&key, "value".into()).unwrap();
//! assert_eq!(kv.get(&key).unwrap(), Some("value".into()));
//! kv.delete(&key).unwrap();
//! ```
//!
//! ## Listing and Filtering
//!
//! ```rust
//! use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
//! let backend = Box::new(MemoryBackend::new());
//! let mut kv = Kv::new(backend);
//! for i in 0..5 as i64 {
//!     let key = (1u64, i, true).to_key();
//!     kv.set(&key, i.into()).unwrap();
//! }
//!
//! // Fetch all records with prefix (1, _, true)
//! let items = kv.list().prefix(&(1u64,)).entries().unwrap();
//! assert!(items.len() >= 1);
//! ```
//!
//! ## Implementing a Backend
//!
//! For custom persistence, implement [`KvBackend`]. See [`backends/mod.rs`](backends/mod.rs) or the SQLite backend for real examples.
//!
//! ## Value Types
//!
//! All values are stored as [`KvValue`] (enum, supports u64, i64, f64, string, bool, null, arrays, objects, binary data).
//!
//! ## JSON Import/Export
//!
//! ```rust
//! use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
//! let mut kv = Kv::new(Box::new(MemoryBackend::new()));
//! let json = kv.dump_json().unwrap();
//! let mut loaded = Kv::from_json_string(Box::new(MemoryBackend::new()), json).unwrap();
//! ```

mod backends;
mod keys;
mod kv_error;
mod kv_value;
mod list_builder;
mod tests;

use std::marker::PhantomData;

pub use crate::backends::{KvBackend, memory_backend::MemoryBackend};
pub use crate::keys::{KvKey, display};
pub use crate::kv_error::{KvError, KvResult};
pub use crate::kv_value::KvValue;
pub use crate::list_builder::KvListBuilder;
pub use keys::IntoKey;
use keys::display::{parse_display_string_to_key, to_display_string};

#[cfg(feature = "sqlite")]
pub use crate::backends::sqlite_backend::SqliteBackend;

/// Main key-value store abstraction.
///
/// Holds a boxed backend and exposes get/set/delete/query APIs.
/// Instantiate with [`Kv::new`], and use [`KvListBuilder`] for advanced listing/filtering.
///
/// # Example
/// ```rust
/// use stupid_simple_kv::{Kv, MemoryBackend, IntoKey};
/// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
/// kv.set(&(123u64, true, "foo"), "bar".into()).unwrap();
/// let out = kv.get(&(123u64, true, "foo")).unwrap();
/// assert_eq!(out, Some("bar".into()));
/// ```
///
pub struct Kv<'a> {
    backend: Box<dyn KvBackend>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Kv<'a> {
    /// Create a new [`Kv`] with the given backend.
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// ```
    pub fn new(backend: Box<dyn KvBackend>) -> Self {
        Self {
            backend,
            _marker: PhantomData,
        }
    }

    /// Retrieve the value for a given key. Returns `Ok(Some(KvValue))` if present, `Ok(None)` if not present.
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// let val = kv.get(&(42u64, "x")).unwrap();
    /// ```
    pub fn get(&self, key: &dyn IntoKey) -> KvResult<Option<KvValue>> {
        let key = key.to_key();
        let pairs = self.backend.get_range(Some(key.clone()), key.successor())?;
        if pairs.is_empty() {
            Ok(None)
        } else {
            let (decoded, _) =
                bincode::decode_from_slice::<KvValue, _>(&pairs[0].1, bincode::config::standard())
                    .map_err(KvError::ValDecodeError)?;
            Ok(Some(decoded))
        }
    }

    /// Set the value for a given key, overwriting it if present.
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// kv.set(&(7u64, true, "foo"), 123i64.into()).unwrap();
    /// ```
    pub fn set(&mut self, key: &dyn IntoKey, value: KvValue) -> KvResult<()> {
        self.set_optional(key, Some(value))
    }

    pub(crate) fn set_optional(
        &mut self,
        key: &dyn IntoKey,
        value: Option<KvValue>,
    ) -> KvResult<()> {
        let key = key.to_key();
        if let Some(v) = value {
            let encoded = bincode::encode_to_vec(v, bincode::config::standard())
                .map_err(KvError::ValEncodeError)?;
            self.backend.set(key, Some(encoded))
        } else {
            // Remove the key completely!
            self.backend.set(key, None)
        }
    }

    /// Delete the value for a given key. Returns the key and previous value if present.
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// let maybe_pair = kv.delete(&(3u64, false));
    /// ```
    pub fn delete(&mut self, key: &dyn IntoKey) -> KvResult<Option<(KvKey, KvValue)>> {
        let made = key.to_key();
        let val = self.get(key)?;
        if let Some(val) = val {
            self.set_optional(key, None)?;
            Ok(Some((made, val)))
        } else {
            Ok(None)
        }
    }

    /// List all entries in the keyspace.
    /// Usually, you should use [`Self::list`] with filters for efficient selects.
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// let all = kv.entries().unwrap();
    /// ```
    pub fn entries(&mut self) -> KvResult<Vec<(KvKey, KvValue)>> {
        KvListBuilder {
            backend: &*self.backend,
            start: None,
            end: None,
            prefix: None,
        }
        .entries()
    }

    /// Build a query for scanning/filtering the key-value space.
    /// Use methods like [`KvListBuilder::prefix`], [`KvListBuilder::start`], [`KvListBuilder::end`] for range scans.
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// // List all keys starting with (1, _)
    /// let results = kv.list().prefix(&(1u64,)).entries().unwrap();
    /// ```
    pub fn list(&'a self) -> KvListBuilder<'a> {
        KvListBuilder::new(&*self.backend)
    }

    /// Dump all keys and values as a pretty, parseable JSON value.
    /// Useful for debugging or migration. Keys are debug-formatted.
    pub fn to_serde_json(&'a mut self) -> KvResult<serde_json::Value> {
        let mut map = serde_json::Map::new();
        for (key, value) in self.entries()? {
            let display = to_display_string(&key.0).ok_or(KvError::KeyDecodeError(format!(
                "Invalid key {key:#?}.\nThis should never happen, please file a bug report."
            )))?;
            map.insert(display, serde_json::Value::from(&value));
        }
        Ok(serde_json::Value::Object(map))
    }

    /// Construct a new `Kv` from a serde-compatible JSON value (from [`to_serde_json`]).
    /// Fails if any key or value is incompatible.
    pub fn from_serde_json(backend: Box<dyn KvBackend>, json: serde_json::Value) -> KvResult<Self> {
        if let Some(obj) = json.as_object() {
            let mut kv = Self::new(backend);
            for (display, value) in obj.iter() {
                let key = parse_display_string_to_key(display).ok_or(KvError::KeyDecodeError(
                    format!("Could not decode JSON key {display} to KvKey."),
                ))?;
                kv.set(&key, KvValue::from(value))?;
            }
            Ok(kv)
        } else {
            Err(KvError::Other(format!(
                "Invalid JSON value while trying to make Kv from serde_json::Value: {json}"
            )))
        }
    }

    /// Dump the entire database to a JSON string.
    /// See [`from_json_string`] for restoring.
    pub fn dump_json(&'a mut self) -> KvResult<String> {
        let json = self.to_serde_json()?;
        Ok(json.to_string())
    }

    /// Restore a `Kv` from a JSON string previously written by [`dump_json`].
    ///
    /// Example:
    /// ```rust
    /// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
    /// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
    /// let json = kv.dump_json().unwrap();
    /// let backend = Box::new(MemoryBackend::new());
    /// let mut loaded = Kv::from_json_string(backend, json).unwrap();
    /// ```
    pub fn from_json_string(backend: Box<dyn KvBackend>, json: String) -> KvResult<Self> {
        let json: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&json)
            .map_err(|e| KvError::Other(format!("serde error parsing json: {e}")))?;
        Self::from_serde_json(backend, serde_json::Value::Object(json))
    }
}
