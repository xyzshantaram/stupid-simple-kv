/// Common types for keys and decode errors.
use std::ops::Deref;

/// Represents a binary key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key(pub Vec<u8>);

impl Deref for Key {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Error during decoding of keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Msg(String),
    UnexpectedEof,
    Invalid,
}
