use crate::keys::decode::KeyDecoder;
use crate::keys::types::{DecodeError, Key};

pub trait KeyEncoder {
    fn encode_key(&self, out: &mut Vec<u8>);
}

pub trait IntoKey {
    fn into_key(self) -> Key;
}

pub trait FromKey: Sized {
    fn from_key(key: &Key) -> Result<Self, DecodeError>;
}
pub trait FromKeySeg: Sized {
    fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError>;
}

// Numeric types (macro)
macro_rules! impl_key_numeric {
    ($([$typ:ty, $as:ty, $which:ident, $tag:expr]),* $(,)?) => {$(
        impl KeyEncoder for $typ {
            #[inline(always)]
            fn encode_key(&self, out: &mut Vec<u8>) {
                out.push($tag);
                out.extend_from_slice(&(<$as>::from(*self)).to_be_bytes());
            }
        }
        impl IntoKey for $typ {
            #[inline(always)]
            fn into_key(self) -> Key {
                let mut out = Vec::with_capacity(9); // 1 tag + 8 bytes max
                self.encode_key(&mut out);
                Key(out)
            }
        }
        impl FromKey for $typ {
            #[inline(always)]
            fn from_key(key: &Key) -> Result<Self, DecodeError> {
                let mut dec = KeyDecoder::new(&key.0);
                Self::from_decoder(&mut dec)
            }
        }
        impl FromKeySeg for $typ {
            #[inline(always)]
            fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError> {
                dec.$which()
                   .map(|v| v as $typ)
                   .ok_or(DecodeError::UnexpectedEof)
            }
        }
    )*};
}

impl_key_numeric! {
    [u8,  u64, next_u64, 0x02],
    [u16, u64, next_u64, 0x02],
    [u32, u64, next_u64, 0x02],
    [u64, u64, next_u64, 0x02],
    [i8,  i64, next_i64, 0x03],
    [i16, i64, next_i64, 0x03],
    [i32, i64, next_i64, 0x03],
    [i64, i64, next_i64, 0x03],
}

// &str and String
impl KeyEncoder for &str {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        out.push(0x01);
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(bytes);
    }
}
impl KeyEncoder for String {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        self.as_str().encode_key(out);
    }
}
impl IntoKey for String {
    #[inline(always)]
    fn into_key(self) -> Key {
        let len = self.len();
        let mut out = Vec::with_capacity(len + 5);
        self.encode_key(&mut out);
        Key(out)
    }
}
impl IntoKey for &str {
    #[inline(always)]
    fn into_key(self) -> Key {
        let len = self.len();
        let mut out = Vec::with_capacity(len + 5);
        self.encode_key(&mut out);
        Key(out)
    }
}
impl FromKey for String {
    #[inline(always)]
    fn from_key(key: &Key) -> Result<Self, DecodeError> {
        let mut dec = KeyDecoder::new(&key.0);
        Self::from_decoder(&mut dec)
    }
}
impl FromKeySeg for String {
    #[inline(always)]
    fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError> {
        dec.next_str()
            .map(|s| s.to_string())
            .ok_or(DecodeError::UnexpectedEof)
    }
}

// bool
impl KeyEncoder for bool {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x05);
        out.push(*self as u8);
    }
}
impl IntoKey for bool {
    #[inline(always)]
    fn into_key(self) -> Key {
        let mut out = Vec::with_capacity(2);
        self.encode_key(&mut out);
        Key(out)
    }
}
impl FromKey for bool {
    #[inline(always)]
    fn from_key(key: &Key) -> Result<Self, DecodeError> {
        let mut dec = KeyDecoder::new(&key.0);
        Self::from_decoder(&mut dec)
    }
}
impl FromKeySeg for bool {
    #[inline(always)]
    fn from_decoder(dec: &mut KeyDecoder) -> Result<Self, DecodeError> {
        dec.next_bool().ok_or(DecodeError::UnexpectedEof)
    }
}

// Key as type itself
impl KeyEncoder for &Key {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.0);
    }
}
impl IntoKey for Key {
    #[inline(always)]
    fn into_key(self) -> Key {
        self
    }
}
impl IntoKey for &Key {
    #[inline(always)]
    fn into_key(self) -> Key {
        self.clone()
    }
}

// Tuple impls
macro_rules! impl_tuple_key {
    ($($T:ident),+) => {
        impl<$($T: KeyEncoder),+> IntoKey for ( $($T,)+ ) {
            #[inline(always)]
            fn into_key(self) -> Key {
                // Estimate an upper bound for capacity (for short strings + ints)
                let mut out = Vec::with_capacity(256);
                #[allow(non_snake_case)]
                let ( $($T,)+ ) = self;
                $($T.encode_key(&mut out);)+
                Key(out)
            }
        }
        impl<$($T: FromKeySeg),+> FromKey for ( $($T,)+ ) {
            #[inline(always)]
            fn from_key(key: &Key) -> Result<Self, DecodeError> {
                let mut dec = KeyDecoder::new(&key.0);
                Ok( ( $($T::from_decoder(&mut dec)? ,)+ ) )
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
