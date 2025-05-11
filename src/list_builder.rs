use crate::{IntoKey, KvBackend, KvKey, KvResult, KvValue};

/// Builder for flexible queries over a key/value backend.
///
/// Use prefix, start, and end keys to define your query range, then call [`KvListBuilder::entries`].
///
/// # Examples
///
/// ```rust
/// // List all pairs with prefix (1, _)
/// let pairs = kv.list().prefix(&(1u64,)).entries().unwrap();
///
/// // Range scan from (99,2) up to (99,5)
/// let result = kv.list().start(&(99u64, 2i64)).end(&(99u64, 5i64)).entries().unwrap();
/// ```
pub struct KvListBuilder<'a> {
    pub(crate) backend: &'a mut Box<dyn KvBackend>,
    pub(crate) prefix: Option<&'a dyn IntoKey>,
    pub(crate) start: Option<&'a dyn IntoKey>,
    pub(crate) end: Option<&'a dyn IntoKey>,
}

impl<'a> KvListBuilder<'a> {
    pub(crate) fn new(backend: &'a mut Box<dyn KvBackend>) -> Self {
        Self {
            backend,
            prefix: None,
            start: None,
            end: None,
        }
    }

    /// Restrict results to the given key prefix.
    pub fn prefix(&mut self, prefix: &'a dyn IntoKey) -> &mut Self {
        self.prefix = Some(prefix);
        self
    }

    /// Start listing at this key (inclusive).
    pub fn start(&mut self, start: &'a dyn IntoKey) -> &mut Self {
        self.start = Some(start);
        self
    }

    /// End listing at this key (exclusive).
    pub fn end(&mut self, end: &'a dyn IntoKey) -> &mut Self {
        self.end = Some(end);
        self
    }

    /// Run the current query and return key-value pairs.
    /// Returns all results matching the filter/prefix/bounds.
    ///
    /// # Errors
    /// Returns an error if the combination of selectors is invalid, or if decoding fails.
    pub fn entries(&self) -> KvResult<Vec<(KvKey, KvValue)>> {
        use crate::KvError;

        let (p, s, e) = (self.prefix, self.start, self.end);

        // Disallow all three present.
        if p.is_some() && s.is_some() && e.is_some() {
            return Err(KvError::InvalidSelector);
        }

        // Helper: convert options to KvKey
        let prefix_key = p.map(|x| x.to_key());
        let start_key = s.map(|x| x.to_key());
        let end_key = e.map(|x| x.to_key());

        let (range_start, range_end) = match (prefix_key, start_key, end_key) {
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
        let items = self.backend.get_range(range_start, range_end)?;

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
