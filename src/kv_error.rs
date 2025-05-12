use std::error::Error;

#[derive(Debug)]
pub enum KvError {
    KeyDecodeError(String),
    InvalidSelector,
    ValEncodeError(bincode::error::EncodeError),
    ValDecodeError(bincode::error::DecodeError),
    ValDowncastError(String),
    Other(String),
    #[cfg(feature = "sqlite")]
    SqliteError(rusqlite::Error),
}

pub type KvResult<T> = Result<T, KvError>;

impl std::fmt::Display for KvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvError::KeyDecodeError(str) => write!(f, "Error decoding key: {str}"),
            KvError::InvalidSelector => write!(
                f,
                "Invalid selector provided - Provide any one or two of start, end, prefix, not all"
            ),
            KvError::ValEncodeError(encode_error) => {
                write!(f, "Error encoding value with bincode: {encode_error}")
            }
            KvError::ValDecodeError(decode_error) => {
                write!(f, "Error decoding value with bincode: {decode_error}")
            }
            KvError::Other(str) => write!(f, "Error during kv op: {str}"),
            KvError::SqliteError(error) => write!(f, "rusqlite error: {error}"),
            KvError::ValDowncastError(s) => write!(f, "Error converting to KvValue: {s}"),
        }
    }
}

impl From<std::cell::BorrowError> for KvError {
    fn from(value: std::cell::BorrowError) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<std::cell::BorrowMutError> for KvError {
    fn from(value: std::cell::BorrowMutError) -> Self {
        Self::Other(value.to_string())
    }
}

impl Error for KvError {}
