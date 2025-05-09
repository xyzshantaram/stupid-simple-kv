//! Key encoding/decoding glue: all traits and helpers for converting to and from binary keys.
pub use encoding::{FromKey, IntoKey, KeyEncoder};
pub use types::{DecodeError, Key};

pub mod decode;
mod encoding;
mod types;