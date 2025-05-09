use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Encode, Decode, Serialize, Deserialize)]
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

#[derive(Debug)]
pub enum KvValueCastError {
    DowncastFail(&'static str),
    SerdeJson(serde_json::Error),
    UnsupportedType,
}

impl fmt::Display for KvValueCastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvValueCastError::DowncastFail(t) => write!(f, "Failed to downcast to {t}"),
            KvValueCastError::SerdeJson(e) => write!(f, "Serde json error: {e}"),
            KvValueCastError::UnsupportedType => write!(f, "Unsupported type for KvValue cast"),
        }
    }
}

impl std::error::Error for KvValueCastError {}

impl KvValue {
    pub fn from_any<T: Serialize + 'static>(v: &T) -> Result<Self, KvValueCastError> {
        use KvValueCastError::*;
        let typeid_bytes = std::any::TypeId::of::<Vec<u8>>();
        let typeid_str = std::any::TypeId::of::<String>();
        let typeid_i64 = std::any::TypeId::of::<i64>();
        let typeid_u64 = std::any::TypeId::of::<u64>();
        let typeid_f64 = std::any::TypeId::of::<f64>();
        let typeid_bool = std::any::TypeId::of::<bool>();
        let t: &dyn std::any::Any = v;
        if t.type_id() == typeid_bytes {
            let b = t.downcast_ref::<Vec<u8>>().ok_or(DowncastFail("Vec<u8>"))?;
            return Ok(KvValue::Binary(b.clone()));
        }
        if t.type_id() == typeid_str {
            let s = t.downcast_ref::<String>().ok_or(DowncastFail("String"))?;
            return Ok(KvValue::String(s.clone()));
        }
        if t.type_id() == typeid_i64 {
            let i = t.downcast_ref::<i64>().ok_or(DowncastFail("i64"))?;
            return Ok(KvValue::I64(*i));
        }
        if t.type_id() == typeid_u64 {
            let i = t.downcast_ref::<u64>().ok_or(DowncastFail("u64"))?;
            return Ok(KvValue::U64(*i));
        }
        if t.type_id() == typeid_f64 {
            let f = t.downcast_ref::<f64>().ok_or(DowncastFail("f64"))?;
            return Ok(KvValue::F64(*f));
        }
        if t.type_id() == typeid_bool {
            let b = t.downcast_ref::<bool>().ok_or(DowncastFail("bool"))?;
            return Ok(KvValue::Bool(*b));
        }
        // Try via serde_json
        let json = serde_json::to_value(v).map_err(SerdeJson)?;
        Ok(match json {
            serde_json::Value::Array(vec) => {
                let mut arr = Vec::with_capacity(vec.len());
                for item in vec {
                    arr.push(KvValue::from_json_value(item));
                }
                KvValue::Array(arr)
            }
            serde_json::Value::Object(map) => {
                let mut obj = BTreeMap::new();
                for (k, v) in map {
                    obj.insert(k, KvValue::from_json_value(v));
                }
                KvValue::Object(obj)
            }
            v => KvValue::from_json_value(v),
        })
    }
    pub fn from_json_value(val: serde_json::Value) -> KvValue {
        match val {
            serde_json::Value::Null => KvValue::Null,
            serde_json::Value::Bool(b) => KvValue::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    KvValue::I64(i)
                } else if let Some(u) = n.as_u64() {
                    KvValue::U64(u)
                } else if let Some(f) = n.as_f64() {
                    KvValue::F64(f)
                } else {
                    KvValue::Null
                }
            }
            serde_json::Value::String(s) => KvValue::String(s),
            serde_json::Value::Array(arr) => {
                KvValue::Array(arr.into_iter().map(KvValue::from_json_value).collect())
            }
            serde_json::Value::Object(map) => KvValue::Object(
                map.into_iter()
                    .map(|(k, v)| (k, KvValue::from_json_value(v)))
                    .collect(),
            ),
        }
    }
    // Convert KvValue back to Rust type T if possible (seriously for values only).
    pub fn to_any<T: serde::de::DeserializeOwned + 'static>(&self) -> Result<T, KvValueCastError> {
        use KvValueCastError::*;
        if let KvValue::Binary(b) = self {
            if std::any::TypeId::of::<T>() == std::any::TypeId::of::<Vec<u8>>() {
                return serde_json::from_value(serde_json::Value::String(STANDARD.encode(b)))
                    .map_err(SerdeJson);
            }
        }
        // Otherwise serialize to JSON value, then to T (via serde)
        let json = self.to_json_value();
        serde_json::from_value(json).map_err(SerdeJson)
    }
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            KvValue::Null => serde_json::Value::Null,
            KvValue::Bool(b) => serde_json::Value::Bool(*b),
            KvValue::I64(i) => serde_json::Value::Number((*i).into()),
            KvValue::U64(u) => serde_json::Value::Number(serde_json::Number::from(*u)),
            KvValue::F64(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            KvValue::String(s) => serde_json::Value::String(s.clone()),
            KvValue::Array(vec) => {
                serde_json::Value::Array(vec.iter().map(|x| x.to_json_value()).collect())
            }
            KvValue::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map.iter() {
                    out.insert(k.clone(), v.to_json_value());
                }
                serde_json::Value::Object(out)
            }
            KvValue::Binary(bytes) => serde_json::Value::String(STANDARD.encode(bytes)),
        }
    }
}
