use crate::{KvBackend, Key, IntoKey};
use std::marker::PhantomData;

pub struct KvListBuilder<'a, B: KvBackend, T> {
    backend: &'a B,
    start: Option<Key>,
    end: Option<Key>,
    prefix: Option<Key>,
    _marker: PhantomData<T>,
}

impl<'a, B: KvBackend, T> KvListBuilder<'a, B, T> {
    pub fn new(backend: &'a B) -> Self {
        Self {
            backend,
            start: None,
            end: None,
            prefix: None,
            _marker: PhantomData,
        }
    }
    pub fn start<K: IntoKey>(mut self, key: K) -> Self {
        self.start = Some(key.into_key());
        self
    }
    pub fn end<K: IntoKey>(mut self, key: K) -> Self {
        self.end = Some(key.into_key());
        self
    }
    pub fn prefix<K: IntoKey>(mut self, prefix: K) -> Self {
        self.prefix = Some(prefix.into_key());
        self
    }
    pub fn iter(self) -> impl Iterator<Item = (Key, T)> + 'a
    where
        T: bincode::Decode<()>,
    {
        let mut keys: Vec<_> = self
            .backend
            .keys()
            .unwrap()
            .filter(|k| {
                let prefix_ok = self.prefix.as_ref().is_none_or(|p| k.0.starts_with(&p.0));
                let start_ok = self.start.as_ref().is_none_or(|s| k >= s);
                let end_ok = self.end.as_ref().is_none_or(|e| k <= e);
                prefix_ok && start_ok && end_ok
            })
            .collect();
        keys.sort();
        keys.into_iter().filter_map(move |k| {
            self.backend
                .get(&k)
                .ok()
                .flatten()
                .and_then(|bytes| {
                    bincode::decode_from_slice(&bytes, bincode::config::standard()).ok()
                })
                .map(|(v, _)| (k, v))
        })
    }
}
