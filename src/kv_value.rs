use serde_json::{Map as JsonMap, Number, Value as JsonValue};
use std::collections::BTreeMap;

use crate::KvError;

/// Any type which can be stored as a value in the key-value store.
///
/// Supports null, bool, i64, u64, f64, String, arrays, objects, and binary blobs.
#[derive(Debug, Clone, PartialEq, PartialOrd, bincode::Encode, bincode::Decode)]
pub enum KvValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Array(Vec<KvValue>),
    Object(BTreeMap<String, KvValue>),
    Binary(Vec<u8>),
}

impl From<()> for KvValue {
    fn from(_: ()) -> Self {
        KvValue::Null
    }
}

impl From<bool> for KvValue {
    fn from(value: bool) -> Self {
        KvValue::Bool(value)
    }
}

impl From<i64> for KvValue {
    fn from(value: i64) -> Self {
        KvValue::I64(value)
    }
}

impl From<u64> for KvValue {
    fn from(value: u64) -> Self {
        KvValue::U64(value)
    }
}

impl From<f64> for KvValue {
    fn from(value: f64) -> Self {
        KvValue::F64(value)
    }
}

impl From<String> for KvValue {
    fn from(value: String) -> Self {
        KvValue::String(value)
    }
}

impl From<&str> for KvValue {
    fn from(value: &str) -> Self {
        KvValue::String(value.to_owned())
    }
}

impl From<Vec<KvValue>> for KvValue {
    fn from(value: Vec<KvValue>) -> Self {
        KvValue::Array(value)
    }
}

impl From<BTreeMap<String, KvValue>> for KvValue {
    fn from(value: BTreeMap<String, KvValue>) -> Self {
        KvValue::Object(value)
    }
}

impl From<Vec<u8>> for KvValue {
    fn from(value: Vec<u8>) -> Self {
        KvValue::Binary(value)
    }
}

impl From<&JsonValue> for KvValue {
    fn from(value: &JsonValue) -> Self {
        match value {
            JsonValue::Null => KvValue::Null,
            JsonValue::Bool(b) => KvValue::Bool(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    KvValue::I64(i)
                } else if let Some(u) = n.as_u64() {
                    KvValue::U64(u)
                } else if let Some(f) = n.as_f64() {
                    KvValue::F64(f)
                } else {
                    KvValue::Null // Shouldn't really happen
                }
            }
            JsonValue::String(s) => KvValue::String(s.clone()),
            JsonValue::Array(arr) => KvValue::Array(arr.iter().map(KvValue::from).collect()),
            JsonValue::Object(obj) => {
                // Check for exact binary tag
                if obj.len() == 2
                    && obj.get("__sskv_bin_value") == Some(&JsonValue::Bool(true))
                    && obj.contains_key("bytes")
                {
                    if let JsonValue::Array(arr) = &obj["bytes"] {
                        let maybe_bytes: Option<Vec<u8>> =
                            arr.iter()
                                .map(|v| {
                                    if let JsonValue::Number(n) = v {
                                        n.as_u64().and_then(|u| {
                                            if u <= 255 { Some(u as u8) } else { None }
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                        if let Some(bytes) = maybe_bytes {
                            return KvValue::Binary(bytes);
                        }
                    }
                }

                // Regular Object fallback
                let map: BTreeMap<String, KvValue> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), KvValue::from(v)))
                    .collect();
                KvValue::Object(map)
            }
        }
    }
}

// From<&KvValue> for JsonValue
impl From<&KvValue> for JsonValue {
    fn from(val: &KvValue) -> Self {
        match val {
            KvValue::Null => JsonValue::Null,
            KvValue::Bool(b) => JsonValue::Bool(*b),
            KvValue::I64(n) => JsonValue::Number(Number::from(*n)),
            KvValue::U64(n) => JsonValue::Number(Number::from(*n)),
            KvValue::F64(f) => Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            KvValue::String(s) => JsonValue::String(s.clone()),
            KvValue::Array(arr) => JsonValue::Array(arr.iter().map(JsonValue::from).collect()),
            KvValue::Object(obj) => {
                let map: JsonMap<String, JsonValue> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::from(v)))
                    .collect();
                JsonValue::Object(map)
            }
            KvValue::Binary(bytes) => {
                let mut map = JsonMap::new();
                map.insert("__sskv_bin_value".to_string(), JsonValue::Bool(true));
                map.insert(
                    "bytes".to_string(),
                    JsonValue::Array(
                        bytes
                            .iter()
                            .map(|b| JsonValue::Number(Number::from(*b)))
                            .collect(),
                    ),
                );
                JsonValue::Object(map)
            }
        }
    }
}

impl TryFrom<KvValue> for () {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::Null => Ok(()),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected Null, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for bool {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::Bool(b) => Ok(b),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected Bool, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for i64 {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::I64(n) => Ok(n),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected I64, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for u64 {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::U64(n) => Ok(n),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected U64, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for f64 {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::F64(n) => Ok(n),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected F64, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for String {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::String(s) => Ok(s),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected String, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for Vec<KvValue> {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::Array(arr) => Ok(arr),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected Array, got {value:?}"
            ))),
        }
    }
}

impl TryFrom<KvValue> for Vec<u8> {
    type Error = KvError;

    fn try_from(value: KvValue) -> Result<Self, Self::Error> {
        match value {
            KvValue::Binary(bytes) => Ok(bytes),
            _ => Err(KvError::ValDowncastError(format!(
                "Expected Binary, got {value:?}"
            ))),
        }
    }
}
