// #![warn(missing_docs)]
#![doc(html_logo_url = "https://avatars3.githubusercontent.com/u/15439811?v=3&s=200",
       html_favicon_url = "https://iorust.github.io/favicon.ico",
       html_root_url = "https://iorust.github.io",
       html_playground_url = "https://play.rust-lang.org",
       issue_tracker_base_url = "https://github.com/iorust/msgp-rust/issues")]

//! Byte message protocol for Rust.

pub fn encode(val: &mut Vec<u8>) -> Vec<u8> {
    let len: usize = val.len();
    assert!(len < 268435456);

    if len < 128 {
        let mut res: Vec<u8> = Vec::with_capacity(len + 1);
        res.push(len as u8);
        res.append(val);
        return res;
    } else if len < 16384 {
        let mut res: Vec<u8> = Vec::with_capacity(len + 2);
        let len = len as u16;
        res.push(((len >> 7) | 0x80u16) as u8);
        res.push((len & 0x7Fu16) as u8);
        res.append(val);
        return res;
    } else if len < 2097152 {
        let mut res: Vec<u8> = Vec::with_capacity(len + 3);
        let len = len as u32;
        res.push((((len >> 14) & 0x7Fu32) as u8) | 0x80u8);
        res.push((((len >> 7) & 0x7Fu32) as u8) | 0x80u8);
        res.push((len & 0x7Fu32) as u8);
        res.append(val);
        return res;
    } else {
        let mut res: Vec<u8> = Vec::with_capacity(len + 4);
        let len = len as u32;
        res.push((((len >> 21) & 0x7Fu32) as u8) | 0x80u8);
        res.push((((len >> 14) & 0x7Fu32) as u8) | 0x80u8);
        res.push((((len >> 7) & 0x7Fu32) as u8) | 0x80u8);
        res.push((len & 0x7Fu32) as u8);
        res.append(val);
        return res;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_len_eq_0() {
        assert_eq!(encode(&mut vec![]), vec![0]);
    }

    #[test]
    fn encode_len_lt_128() {
        assert_eq!(encode(&mut vec![0x1u8, 0x2u8]), vec![0x2u8, 0x1u8, 0x2u8]);

        let mut res = Vec::with_capacity(127 + 1);
        res.extend_from_slice(&[0x7fu8]);
        res.append(&mut vec![0xffu8; 127]);
        assert_eq!(encode(&mut vec![0xffu8; 127]), res);
    }

    #[test]
    fn encode_len_eq_128() {
        let mut res = Vec::with_capacity(128 + 2);
        res.extend_from_slice(&[0x81u8, 0x00u8]);
        res.append(&mut vec![0xffu8; 128]);
        assert_eq!(encode(&mut vec![0xffu8; 128]), res);
    }

    #[test]
    fn encode_len_lt_16384() {
        let mut res = Vec::with_capacity(129 + 2);
        res.extend_from_slice(&[0x81u8, 0x01u8]);
        res.append(&mut vec![0xffu8; 129]);
        assert_eq!(encode(&mut vec![0xffu8; 129]), res);

        let mut res = Vec::with_capacity(16383 + 2);
        res.extend_from_slice(&[0xffu8, 0x7fu8]);
        res.append(&mut vec![0xffu8; 16383]);
        assert_eq!(encode(&mut vec![0xffu8; 16383]), res);
    }

    #[test]
    fn encode_len_eq_16384() {
        let mut res = Vec::with_capacity(16384 + 3);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x00u8]);
        res.append(&mut vec![0xffu8; 16384]);
        assert_eq!(encode(&mut vec![0xffu8; 16384]), res);
    }

    #[test]
    fn encode_len_lt_2097152() {
        let mut res = Vec::with_capacity(16385 + 3);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x01u8]);
        res.append(&mut vec![0xffu8; 16385]);
        assert_eq!(encode(&mut vec![0xffu8; 16385]), res);

        let mut res = Vec::with_capacity(2097151 + 3);
        res.extend_from_slice(&[0xffu8, 0xffu8, 0x7fu8]);
        res.append(&mut vec![0xffu8; 2097151]);
        assert_eq!(encode(&mut vec![0xffu8; 2097151]), res);
    }

    #[test]
    fn encode_len_eq_2097152() {
        let mut res = Vec::with_capacity(2097152 + 4);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x80u8, 0x00u8]);
        res.append(&mut vec![0xffu8; 2097152]);
        assert_eq!(encode(&mut vec![0xffu8; 2097152]), res);
    }

    #[test]
    fn encode_len_lt_268435456() {
        let mut res = Vec::with_capacity(2097153 + 4);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x80u8, 0x01u8]);
        res.append(&mut vec![0xffu8; 2097153]);
        assert_eq!(encode(&mut vec![0xffu8; 2097153]), res);

        let mut res = Vec::with_capacity(268435455 + 4);
        res.extend_from_slice(&[0xffu8, 0xffu8, 0xffu8, 0x7fu8]);
        res.append(&mut vec![0xffu8; 268435455]);
        assert_eq!(encode(&mut vec![0xffu8; 268435455]), res);
    }
}
