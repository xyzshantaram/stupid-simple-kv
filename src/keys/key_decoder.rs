use crate::keys::key_segment::KeySegmentTag;
use crate::{KvError, KvKey};

pub struct KeyDecoder<'a> {
    rem: &'a [u8],
}

impl<'a> KeyDecoder<'a> {
    pub fn new(rem: &'a [u8]) -> Self {
        Self { rem }
    }

    pub fn next_str(&mut self) -> Option<&'a str> {
        if self.rem.len() < 9 || self.rem[0] != KeySegmentTag::String as u8 {
            return None;
        }

        let len = usize::from_be_bytes(self.rem[1..9].try_into().ok()?);
        if self.rem.len() < 9 + len {
            return None;
        }

        let out = str::from_utf8(&self.rem[9..len + 9]).ok()?;
        self.rem = &self.rem[9 + len..];
        Some(out)
    }

    pub fn next_bool(&mut self) -> Option<bool> {
        if self.rem.len() < 2 || self.rem[0] != KeySegmentTag::Bool as u8 {
            return None;
        }
        let byte = self.rem[1];
        self.rem = &self.rem[2..];
        Some(byte != 0)
    }

    pub fn next_i64(&mut self) -> Option<i64> {
        if self.rem.len() < 9 || self.rem[0] != KeySegmentTag::I64 as u8 {
            return None;
        }
        let bytes: [u8; 8] = self.rem[1..9].try_into().ok()?;
        let int = i64::from_be_bytes(bytes);
        self.rem = &self.rem[9..];
        Some(int)
    }

    pub fn next_u64(&mut self) -> Option<u64> {
        if self.rem.len() < 9 || self.rem[0] != KeySegmentTag::U64 as u8 {
            return None;
        }
        let bytes: [u8; 8] = self.rem[1..9].try_into().ok()?;
        let num = u64::from_be_bytes(bytes);
        self.rem = &self.rem[9..];
        Some(num)
    }
}

pub trait FromKvKey<'a>: Sized {
    fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self>;
}

impl<'a> FromKvKey<'a> for i64 {
    fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self> {
        decoder.next_i64()
    }
}

impl<'a> FromKvKey<'a> for u64 {
    fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self> {
        decoder.next_u64()
    }
}

impl<'a> FromKvKey<'a> for bool {
    fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self> {
        decoder.next_bool()
    }
}

impl<'a> FromKvKey<'a> for &'a str {
    fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self> {
        decoder.next_str()
    }
}

impl<'a> FromKvKey<'a> for String {
    fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self> {
        decoder.next_str().map(String::from)
    }
}

macro_rules! impl_key_decode_for_tuple {
    ($($name:ident),+) => {
        impl<'a, $($name),+> FromKvKey<'a> for ($($name,)+)
        where
            $($name: FromKvKey<'a>),+
        {
            fn from_kv_key(decoder: &mut KeyDecoder<'a>) -> Option<Self> {
                Some((
                    $(
                        <$name as FromKvKey>::from_kv_key(decoder)?,
                    )+
                ))
            }
        }
    };
}

impl_key_decode_for_tuple!(A);
impl_key_decode_for_tuple!(A, B);
impl_key_decode_for_tuple!(A, B, C);
impl_key_decode_for_tuple!(A, B, C, D);
impl_key_decode_for_tuple!(A, B, C, D, E);
impl_key_decode_for_tuple!(A, B, C, D, E, F);
impl_key_decode_for_tuple!(A, B, C, D, E, F, G);
impl_key_decode_for_tuple!(A, B, C, D, E, F, G, H);
impl_key_decode_for_tuple!(A, B, C, D, E, F, G, H, I);

macro_rules! impl_kv_key_try_from_tuple {
    ($($name:ident),+) => {
        impl<$($name: for<'a> FromKvKey<'a>),+> TryFrom<KvKey> for ($($name,)+) {
            type Error = KvError;
            fn try_from(key: KvKey) -> Result<Self, Self::Error> {
                let mut decoder = KeyDecoder::new(&key.0);
                $(
                    #[allow(non_snake_case)]
                    let $name = <$name as FromKvKey>::from_kv_key(&mut decoder)
                        .ok_or_else(|| KvError::KeyDecodeError(
                            format!("Failed to decode key segment \"{}\"", stringify!($name)))
                        )?;
                )+
                Ok(($($name,)+))
            }
        }
    }
}

impl_kv_key_try_from_tuple!(A);
impl_kv_key_try_from_tuple!(A, B);
impl_kv_key_try_from_tuple!(A, B, C);
impl_kv_key_try_from_tuple!(A, B, C, D);
impl_kv_key_try_from_tuple!(A, B, C, D, E);
impl_kv_key_try_from_tuple!(A, B, C, D, E, F);
impl_kv_key_try_from_tuple!(A, B, C, D, E, F, G);
impl_kv_key_try_from_tuple!(A, B, C, D, E, F, G, H);
impl_kv_key_try_from_tuple!(A, B, C, D, E, F, G, H, I);
