use crate::KvBackend;
use std::marker::PhantomData;

pub struct KvListBuilder<'a, B: KvBackend, T> {
    backend: &'a B,
    start: Option<Vec<u8>>,
    end: Option<Vec<u8>>,
    prefix: Option<Vec<u8>>,
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

    pub fn start(mut self, key: &[u8]) -> Self {
        self.start = Some(key.to_vec());
        self
    }
    pub fn end(mut self, key: &[u8]) -> Self {
        self.end = Some(key.to_vec());
        self
    }
    pub fn prefix(mut self, prefix: &[u8]) -> Self {
        self.prefix = Some(prefix.to_vec());
        self
    }

    pub fn iter(self) -> impl Iterator<Item = (Vec<u8>, T)> + 'a
    where
        T: bincode::Decode<()>,
    {
        let mut keys: Vec<_> = self
            .backend
            .keys()
            .unwrap()
            .filter(|k| {
                let prefix_ok = self.prefix.as_ref().is_none_or(|p| k.starts_with(p));
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
