use std::str;

/// Decoder for parsing binary keys (FDB/Deno-style) into primitives.
pub struct KeyDecoder<'a> {
    rem: &'a [u8],
}

impl<'a> KeyDecoder<'a> {
    pub fn new(rem: &'a [u8]) -> Self {
        Self { rem }
    }
    pub fn next_str(&mut self) -> Option<&'a str> {
        if self.rem.len() < 5 || self.rem[0] != 0x01 {
            return None;
        }
        let len = u32::from_be_bytes(self.rem[1..5].try_into().unwrap()) as usize;
        if self.rem.len() < 5 + len {
            return None;
        }
        let out = str::from_utf8(&self.rem[5..5 + len]).ok()?;
        self.rem = &self.rem[5 + len..];
        Some(out)
    }
    pub fn next_u64(&mut self) -> Option<u64> {
        if self.rem.len() < 9 || self.rem[0] != 0x02 {
            return None;
        }
        let n = u64::from_be_bytes(self.rem[1..9].try_into().unwrap());
        self.rem = &self.rem[9..];
        Some(n)
    }
    pub fn next_i64(&mut self) -> Option<i64> {
        if self.rem.len() < 9 || self.rem[0] != 0x03 {
            return None;
        }
        let n = i64::from_be_bytes(self.rem[1..9].try_into().unwrap());
        self.rem = &self.rem[9..];
        Some(n)
    }
    pub fn next_bool(&mut self) -> Option<bool> {
        if self.rem.len() < 2 || self.rem[0] != 0x05 {
            return None;
        }
        let b = match self.rem[1] {
            0 => false,
            1 => true,
            _ => return None,
        };
        self.rem = &self.rem[2..];
        Some(b)
    }
}
