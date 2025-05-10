use super::{KvKey, key_segment::KeySegmentTag};
use std::str::FromStr;

pub fn to_display_string(mut rem: &[u8]) -> Option<String> {
    let mut parts = Vec::new();
    while !rem.is_empty() {
        if rem[0] == KeySegmentTag::String as u8 {
            if rem.len() < 9 {
                return None;
            }
            let len = usize::from_be_bytes(rem[1..9].try_into().ok()?);
            if rem.len() < 9 + len {
                return None;
            }
            let s = std::str::from_utf8(&rem[9..9 + len]).ok()?;
            // Escape colons not already escaped
            let mut escaped = String::with_capacity(s.len());
            let mut chars = s.chars().peekable();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    if chars.peek() == Some(&':') {
                        // keep the backslash to allow for \: (escaped colon)
                        escaped.push('\\');
                        // next iteration will handle the colon
                    }
                    // else, ignore the backslash per your instructions
                } else if c == ':' {
                    escaped.push_str("\\:");
                } else {
                    escaped.push(c);
                }
            }
            parts.push(escaped);
            rem = &rem[9 + len..];
        } else if rem[0] == KeySegmentTag::Bool as u8 {
            if rem.len() < 2 {
                return None;
            }
            let b = rem[1] != 0;
            parts.push(b.to_string());
            rem = &rem[2..];
        } else if rem[0] == KeySegmentTag::I64 as u8 {
            if rem.len() < 9 {
                return None;
            }
            let bytes: [u8; 8] = rem[1..9].try_into().ok()?;
            let n = i64::from_be_bytes(bytes);
            if n >= 0 {
                parts.push(format!("{n}i"));
            } else {
                parts.push(format!("-{}", -n));
            }
            rem = &rem[9..];
        } else if rem[0] == KeySegmentTag::U64 as u8 {
            if rem.len() < 9 {
                return None;
            }
            let bytes: [u8; 8] = rem[1..9].try_into().ok()?;
            let n = u64::from_be_bytes(bytes);
            parts.push(n.to_string());
            rem = &rem[9..];
        } else {
            // Unknown tag - bail out
            return None;
        }
    }
    Some(parts.join(":"))
}

pub fn parse_display_string_to_key(display: &str) -> Option<KvKey> {
    let mut key = KvKey::new();
    let mut buf = String::with_capacity(display.len());
    let mut chars = display.chars().peekable();

    let mut parts = Vec::new();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(':') = chars.peek() {
                buf.push(':');
                chars.next(); // consume the colon
            }
        } else if c == ':' {
            parts.push(std::mem::take(&mut buf));
        } else {
            buf.push(c);
        }
    }
    parts.push(buf);

    for part in parts {
        // Try bool
        if part == "true" {
            key.push(&true);
            continue;
        }
        if part == "false" {
            key.push(&false);
            continue;
        }
        // i64 negative: -digits (no trailing i)
        if let Some(rest) = part.strip_prefix('-') {
            if rest.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(num) = i64::from_str(&format!("-{rest}")) {
                    key.push(&num);
                    continue;
                }
            }
        }
        // i64 positive: digits + 'i'
        if part.ends_with('i') && part.len() > 1 {
            let digits = &part[..part.len() - 1];
            if digits.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(num) = i64::from_str(digits) {
                    key.push(&num);
                    continue;
                }
            }
        }
        // u64: only digits
        if part.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(num) = u64::from_str(&part) {
                key.push(&num);
                continue;
            }
        }
        // Otherwise treat as string
        key.push(&part);
    }

    Some(key)
}
