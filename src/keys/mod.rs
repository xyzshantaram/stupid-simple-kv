use key_segment::KeySegment;
pub mod display;
mod key_decoder;
mod key_segment;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct KvKey(pub(crate) Vec<u8>);

impl KvKey {
    fn new() -> Self {
        Self(Vec::with_capacity(128))
    }

    fn push(&mut self, part: &dyn KeySegment) {
        part.encode_into(&mut self.0);
    }

    pub fn starts_with(&self, key: &KvKey) -> bool {
        self.0.starts_with(&key.0)
    }

    /// Returns the smallest key that is strictly greater than this one.
    /// Useful for exclusive upper bounds in range queries.
    pub fn successor(&self) -> Option<KvKey> {
        let mut bytes = self.0.clone();
        for i in (0..bytes.len()).rev() {
            if bytes[i] != 0xFF {
                bytes[i] += 1;
                bytes.truncate(i + 1); // next higher key, all bytes after that don't matter
                return Some(KvKey(bytes));
            }
            // else, keep looking left
        }
        // All bytes were 0xFF, no higher key possible
        None
    }
}

pub trait IntoKey {
    fn to_key(&self) -> KvKey;
}

impl IntoKey for u64 {
    fn to_key(&self) -> KvKey {
        let mut key = KvKey::new();
        key.push(self);
        key
    }
}

impl IntoKey for i64 {
    fn to_key(&self) -> KvKey {
        let mut key = KvKey::new();
        key.push(self);
        key
    }
}

impl IntoKey for String {
    fn to_key(&self) -> KvKey {
        let mut key = KvKey::new();
        key.push(self);
        key
    }
}

impl IntoKey for bool {
    fn to_key(&self) -> KvKey {
        let mut key = KvKey::new();
        key.push(self);
        key
    }
}

impl IntoKey for &str {
    fn to_key(&self) -> KvKey {
        let mut key = KvKey::new();
        key.push(self);
        key
    }
}

impl IntoKey for KvKey {
    fn to_key(&self) -> KvKey {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{KvResult, keys::IntoKey};

    // These macros/traits must already be in your crate:
    //   - impl_kv_key_for_tuple! (for IntoKey)
    //   - impl_kvkey_tryfrom_tuple! (for TryFrom)
    //   - KeySegment for u64, String, bool, i64, etc.
    //   - KeySegmentDecode for those types

    #[test]
    fn roundtrip_one_tuple() -> KvResult<()> {
        let tup = (42u64,);
        let key = tup.to_key();
        let out: (u64,) = key.try_into()?;
        assert_eq!(tup, out);
        Ok(())
    }

    #[test]
    fn roundtrip_simple_tuple() -> KvResult<()> {
        let tup = (1234u64, String::from("hello"));
        let key = tup.clone().to_key();
        let out: (u64, String) = key.try_into()?;
        assert_eq!(tup, out);
        Ok(())
    }

    #[test]
    fn roundtrip_longer_tuple() -> KvResult<()> {
        let tup = (19u64, -5i64, "foo", true, false);
        let key = tup.clone().to_key();
        let out: (u64, i64, String, bool, bool) = key.try_into()?;
        assert_eq!((tup.0, tup.1, tup.2.to_owned(), tup.3, tup.4), out);
        Ok(())
    }

    #[test]
    fn roundtrip_tuple_tryfrom() -> KvResult<()> {
        let tup = (7u64, "hello world", true);
        let key = tup.clone().to_key();
        let out: (u64, String, bool) = key.try_into()?;
        assert_eq!((tup.0, tup.1.to_owned(), tup.2), out);
        Ok(())
    }

    #[test]
    fn decode_error_wrong_type() -> KvResult<()> {
        let tup = (237u64, false);
        let key = tup.to_key();
        // Attempt to decode as (bool, u64): type order/encoding mismatch
        let out: KvResult<(bool, u64)> = key.try_into();
        assert!(out.is_err());
        Ok(())
    }

    #[test]
    fn decode_error_wrong_length() -> KvResult<()> {
        let tup = (55u64, "xyz");
        let key = tup.to_key();
        // Attempt to decode as a 3-tuple (should fail)
        let out: KvResult<(u64, String, bool)> = key.try_into();
        assert!(out.is_err());
        Ok(())
    }

    #[test]
    fn roundtrip_with_strings() -> KvResult<()> {
        let tup = (999u64, "potato", "apple", true);
        let key = tup.clone().to_key();
        let out: (u64, String, String, bool) = key.try_into()?;
        assert_eq!((tup.0, tup.1.to_owned(), tup.2.to_owned(), tup.3), out);
        Ok(())
    }

    #[test]
    fn roundtrip_false_bool() -> KvResult<()> {
        let tup = (0u64, false, "z");
        let key = tup.clone().to_key();
        let out: (u64, bool, String) = key.try_into()?;
        assert_eq!((tup.0, tup.1, tup.2.to_owned()), out);
        Ok(())
    }
}
