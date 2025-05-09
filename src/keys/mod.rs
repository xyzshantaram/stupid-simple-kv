//! Key encoding/decoding glue: all traits and helpers for converting to and from binary keys.
pub use key_traits::{FromKey, IntoKey};
pub use types::{DecodeError, Key};

// Export KeyEncoder/KeyDecoder still from encode/decode as internal only, not public API
pub mod decode;
pub mod encode;
mod key_traits;
mod types;
