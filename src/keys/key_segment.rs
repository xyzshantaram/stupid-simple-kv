use crate::keys::{IntoKey, KvKey};

#[repr(u8)]
pub(crate) enum KeySegmentTag {
    U64 = 0x01,
    I64 = 0x02,
    Bool = 0x03,
    String = 0x04,
}

pub trait KeySegment {
    fn encode_into(&self, out: &mut Vec<u8>);
}

impl KeySegment for u64 {
    fn encode_into(&self, out: &mut Vec<u8>) {
        out.push(KeySegmentTag::U64 as u8);
        out.extend_from_slice(&self.to_be_bytes());
    }
}

impl KeySegment for i64 {
    fn encode_into(&self, out: &mut Vec<u8>) {
        out.push(KeySegmentTag::I64 as u8);
        out.extend_from_slice(&self.to_be_bytes());
    }
}

impl KeySegment for bool {
    fn encode_into(&self, out: &mut Vec<u8>) {
        out.push(KeySegmentTag::Bool as u8);
        out.push(*self as u8);
    }
}

impl KeySegment for String {
    fn encode_into(&self, out: &mut Vec<u8>) {
        out.push(KeySegmentTag::String as u8);
        out.extend_from_slice(&(self.len() as u64).to_be_bytes());
        out.extend_from_slice(self.as_bytes());
    }
}

impl KeySegment for &str {
    fn encode_into(&self, out: &mut Vec<u8>) {
        out.push(KeySegmentTag::String as u8);
        out.extend_from_slice(&(self.len() as u64).to_be_bytes());
        out.extend_from_slice(self.as_bytes());
    }
}

macro_rules! impl_key_encode_for_tuple {
    ($($name:ident),+) => {
        impl<$($name: KeySegment),+> IntoKey for ($($name,)+) {
            fn to_key(&self) -> KvKey {
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                let mut key = KvKey::new();
                $(
                    key.push($name);
                )+
                key
            }
        }
    };
}

impl_key_encode_for_tuple!(A);
impl_key_encode_for_tuple!(A, B);
impl_key_encode_for_tuple!(A, B, C);
impl_key_encode_for_tuple!(A, B, C, D);
impl_key_encode_for_tuple!(A, B, C, D, E);
impl_key_encode_for_tuple!(A, B, C, D, E, F);
impl_key_encode_for_tuple!(A, B, C, D, E, F, G);
impl_key_encode_for_tuple!(A, B, C, D, E, F, G, H);
impl_key_encode_for_tuple!(A, B, C, D, E, F, G, H, I);
