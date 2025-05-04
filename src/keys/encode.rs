pub trait KeyEncoder {
    fn encode_key(&self, out: &mut Vec<u8>);
}

impl KeyEncoder for &str {
    fn encode_key(&self, out: &mut Vec<u8>) {
        let bytes = self.as_bytes();
        out.push(0x01);
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        out.extend_from_slice(bytes);
    }
}
impl KeyEncoder for String {
    fn encode_key(&self, out: &mut Vec<u8>) {
        self.as_str().encode_key(out);
    }
}
impl KeyEncoder for u64 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&self.to_be_bytes());
    }
}
impl KeyEncoder for i64 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&self.to_be_bytes());
    }
}

impl KeyEncoder for i8 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&(i64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for i16 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&(i64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for i32 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x03);
        out.extend_from_slice(&(i64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for u8 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&(u64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for u16 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&(u64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for u32 {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x02);
        out.extend_from_slice(&(u64::from(*self)).to_be_bytes());
    }
}
impl KeyEncoder for bool {
    fn encode_key(&self, out: &mut Vec<u8>) {
        out.push(0x05);
        out.push(if *self { 1 } else { 0 });
    }
}


