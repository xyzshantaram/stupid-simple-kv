use crate::keys::decode::KeyDecoder;
use crate::keys::encode::KeyEncoder;
use crate::keys::types::{DecodeError, Key};

pub trait IntoKey {
    fn into_key(self) -> Key;
}

pub trait FromKey: Sized {
    fn from_key(key: &Key) -> Result<Self, DecodeError>;
}

/// Internal: for decoding a segment from KeyDecoder (primitive/string only)
pub trait FromKeySeg: Sized {
    fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError>;
}

// Primitive impls
macro_rules! impl_key_traits {
    ($typ:ty, next: $next:ident, from: $to:ty) => {
        impl IntoKey for $typ {
            fn into_key(self) -> Key {
                let mut out = Vec::new();
                KeyEncoder::encode_key(&self, &mut out);
                Key(out)
            }
        }
        impl FromKey for $typ {
            fn from_key(key: &Key) -> Result<Self, DecodeError> {
                let mut dec = KeyDecoder::new(&key.0);
                FromKeySeg::from_decoder(&mut dec)
            }
        }
        impl FromKeySeg for $typ {
            fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError> {
                dec.$next()
                    .map(|v| v as $typ)
                    .ok_or(DecodeError::UnexpectedEof)
            }
        }
    };
}
impl_key_traits!(u8, next: next_u64, from: u64);
impl_key_traits!(u16, next: next_u64, from: u64);
impl_key_traits!(u32, next: next_u64, from: u64);
impl_key_traits!(u64, next: next_u64, from: u64);
impl_key_traits!(i8, next: next_i64, from: i64);
impl_key_traits!(i16, next: next_i64, from: i64);
impl_key_traits!(i32, next: next_i64, from: i64);
impl_key_traits!(i64, next: next_i64, from: i64);

impl IntoKey for bool {
    fn into_key(self) -> Key {
        let mut out = Vec::new();
        KeyEncoder::encode_key(&self, &mut out);
        Key(out)
    }
}
impl FromKey for bool {
    fn from_key(key: &Key) -> Result<Self, DecodeError> {
        let mut dec = KeyDecoder::new(&key.0);
        FromKeySeg::from_decoder(&mut dec)
    }
}
impl FromKeySeg for bool {
    fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError> {
        dec.next_bool().ok_or(DecodeError::UnexpectedEof)
    }
}

impl IntoKey for String {
    fn into_key(self) -> Key {
        let mut out = Vec::new();
        KeyEncoder::encode_key(&self, &mut out);
        Key(out)
    }
}
impl IntoKey for &str {
    fn into_key(self) -> Key {
        let mut out = Vec::new();
        KeyEncoder::encode_key(&self, &mut out);
        Key(out)
    }
}
impl FromKey for String {
    fn from_key(key: &Key) -> Result<Self, DecodeError> {
        let mut dec = KeyDecoder::new(&key.0);
        FromKeySeg::from_decoder(&mut dec)
    }
}
impl FromKeySeg for String {
    fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError> {
        dec.next_str()
            .map(|s| s.to_string())
            .ok_or(DecodeError::UnexpectedEof)
    }
}

// Allow passing Key or &Key directly for less boilerplate in Kv methods
impl IntoKey for Key {
    fn into_key(self) -> Key {
        self
    }
}
impl IntoKey for &Key {
    fn into_key(self) -> Key {
        self.clone()
    }
}

macro_rules! impl_tuple_key {
    ($( $T:ident ),+) => {
        impl<$( $T: IntoKey ),+> IntoKey for ( $( $T, )+ ) {
            fn into_key(self) -> Key {
                #[allow(non_snake_case)]
                let ( $( $T, )+ ) = self;
                let mut out = Vec::new();
                $( out.extend($T.into_key().0); )+
                Key(out)
            }
        }
        impl<$( $T: FromKeySeg ),+> FromKey for ( $( $T, )+ ) {
            fn from_key(key: &Key) -> Result<Self, DecodeError> {
                let mut dec = KeyDecoder::new(&key.0);
                Ok(( $( $T::from_decoder(&mut dec)? ),+ , )) // Trailing comma!
            }
        }
    };
}
impl_tuple_key!(A);
impl_tuple_key!(A, B);
impl_tuple_key!(A, B, C);
impl_tuple_key!(A, B, C, D);
impl_tuple_key!(A, B, C, D, E);
impl_tuple_key!(A, B, C, D, E, F);
impl_tuple_key!(A, B, C, D, E, F, G);
impl_tuple_key!(A, B, C, D, E, F, G, H);
impl_tuple_key!(A, B, C, D, E, F, G, H, I);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple_key!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
