/// Encoding primitives for building binary keys.
pub trait KeyEncoder {
    fn encode_key(&self, out: &mut Vec<u8>);
}

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
impl KeyEncoder for u64 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&self.to_be_bytes());
    }
}
impl KeyEncoder for i64 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&self.to_be_bytes());
    }
}

impl KeyEncoder for i8 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&(i64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for i16 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&(i64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for i32 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&(i64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for u8 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&(u64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for u16 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&(u64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for u32 {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&(u64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for bool {
    #[inline(always)]
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x05);
        out.push(if *self { 1 } else { 0 });
    }
}
