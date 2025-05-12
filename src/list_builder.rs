use std::{cell::RefCell, rc::Rc};

use crate::{IntoKey, KvBackend, KvKey, KvResult, KvValue};

/// Builder for flexible queries over a key/value backend.
///
/// Use prefix, start, and end keys to define your query range, then call [`KvListBuilder::entries`].
///
/// # Examples
///
/// ```rust
/// use stupid_simple_kv::{Kv, MemoryBackend, KvValue, IntoKey};
/// let mut kv = Kv::new(Box::new(MemoryBackend::new()));
///
/// // List all pairs with prefix (1, _)
/// let pairs = kv.list().prefix(&(1u64,)).entries().unwrap();
///
/// // Range scan from (99,2) up to (99,5)
/// let result = kv.list().start(&(99u64, 2i64)).end(&(99u64, 5i64)).entries().unwrap();
/// ```
pub struct KvListBuilder {
    pub(crate) backend: Rc<RefCell<Box<dyn KvBackend>>>,
    pub(crate) prefix: Option<KvKey>,
    pub(crate) start: Option<KvKey>,
    pub(crate) end: Option<KvKey>,
}

impl KvListBuilder {
    pub(crate) fn new(backend: Rc<RefCell<Box<dyn KvBackend>>>) -> Self {
        Self {
            backend,
            prefix: None,
            start: None,
            end: None,
        }
    }

    /// Restrict results to the given key prefix.
    pub fn prefix(&mut self, prefix: &dyn IntoKey) -> &mut Self {
        self.prefix = Some(prefix.to_key());
        self
    }

    /// Start listing at this key (inclusive).
    pub fn start(&mut self, start: &dyn IntoKey) -> &mut Self {
        self.start = Some(start.to_key());
        self
    }

    /// End listing at this key (exclusive).
    pub fn end(&mut self, end: &dyn IntoKey) -> &mut Self {
        self.end = Some(end.to_key());
        self
    }

    /// Run the current query and return key-value pairs.
    /// Returns all results matching the filter/prefix/bounds.
    ///
    /// # Errors
    /// Returns an error if the combination of selectors is invalid, or if decoding fails.
    pub fn entries(&self) -> KvResult<Vec<(KvKey, KvValue)>> {
        use crate::KvError;

        // Disallow all three present.
        if self.prefix.is_some() && self.start.is_some() && self.end.is_some() {
            return Err(KvError::InvalidSelector);
        }

        let (range_start, range_end) =
            match (self.prefix.clone(), self.start.clone(), self.end.clone()) {
                (Some(prefix), None, None) => {
                    let end = prefix.successor();
                    (Some(prefix), end)
                }
                (None, Some(start), None) => (Some(start), None),
                (None, None, Some(end)) => (None, Some(end)),
                (Some(_prefix), Some(start), None) => (Some(start), None), // start wins
                (Some(prefix), None, Some(end)) => (Some(prefix), Some(end)),
                (None, Some(start), Some(end)) => (Some(start), Some(end)),
                (None, None, None) => (None, None),
                _ => return Err(KvError::InvalidSelector),
            };

        // Fetch the range (unbounded if end is None)
        let items = self
            .backend
            .try_borrow()?
            .get_range(range_start, range_end)?;

        let mut result = Vec::with_capacity(items.len());
        for (k, v) in items {
            let (decoded, _consumed) =
                bincode::decode_from_slice::<KvValue, _>(&v, bincode::config::standard())
                    .map_err(KvError::ValDecodeError)?;
            result.push((k, decoded));
        }
        Ok(result)
    }
}
