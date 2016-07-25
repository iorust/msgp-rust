// #![warn(missing_docs)]
#![doc(html_logo_url = "https://avatars3.githubusercontent.com/u/15439811?v=3&s=200",
       html_favicon_url = "https://iorust.github.io/favicon.ico",
       html_root_url = "https://iorust.github.io",
       html_playground_url = "https://play.rust-lang.org",
       issue_tracker_base_url = "https://github.com/iorust/msgp-rust/issues")]

//! Byte message protocol for Rust.

use std::ptr;
use std::io::{Result, Error, ErrorKind};

pub fn encode(val: &[u8]) -> Vec<u8> {
    let len: usize = val.len();
    assert!(len < 268435456);

    if len < 128 {
        let mut res: Vec<u8> = Vec::with_capacity(len + 1);
        res.push(len as u8);
        copy_to_vec(val, &mut res, 1, 0, val.len());
        return res;
    } else if len < 16384 {
        let mut res: Vec<u8> = Vec::with_capacity(len + 2);
        let len = len as u16;
        res.push(((len >> 7) | 0x80u16) as u8);
        res.push((len & 0x7Fu16) as u8);
        copy_to_vec(val, &mut res, 2, 0, val.len());
        return res;
    } else if len < 2097152 {
        let mut res: Vec<u8> = Vec::with_capacity(len + 3);
        let len = len as u32;
        res.push((((len >> 14) & 0x7Fu32) as u8) | 0x80u8);
        res.push((((len >> 7) & 0x7Fu32) as u8) | 0x80u8);
        res.push((len & 0x7Fu32) as u8);
        copy_to_vec(val, &mut res, 3, 0, val.len());
        return res;
    } else {
        let mut res: Vec<u8> = Vec::with_capacity(len + 4);
        let len = len as u32;
        res.push((((len >> 21) & 0x7Fu32) as u8) | 0x80u8);
        res.push((((len >> 14) & 0x7Fu32) as u8) | 0x80u8);
        res.push((((len >> 7) & 0x7Fu32) as u8) | 0x80u8);
        res.push((len & 0x7Fu32) as u8);
        copy_to_vec(val, &mut res, 4, 0, val.len());
        return res;
    }
}

pub fn decode(val: &[u8]) -> Option<Vec<u8>> {
    let src_len = val.len();
    if src_len == 0 {
        return None;
    }

    if let Ok(res) = parse_buffer(val, 0) {
        if res.1 == 0 || res.1 > src_len {
            return None;
        }
        let len: usize = res.1 - res.0;
        let mut buf: Vec<u8> = Vec::with_capacity(len);
        copy_to_vec(val, &mut buf, 0, res.0, len);
        return Some(buf);
    }
    return None;
}

/// A streaming Msgp decoder.
#[derive(Debug)]
pub struct Decoder {
    pos: usize,
    msg_start: usize,
    msg_end: usize,
    buf: Vec<u8>,
    res: Vec<Vec<u8>>,
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            pos: 0,
            msg_start: 0,
            msg_end: 0,
            buf: Vec::new(),
            res: Vec::with_capacity(8),
        }
    }

    pub fn feed(&mut self, buf: &[u8]) -> Result<usize> {
        self.buf.extend_from_slice(buf);
        self.parse()
    }

    /// Reads a decoded massage buffer, will return `None` if no buffer decoded.
    pub fn read(&mut self) -> Option<Vec<u8>> {
        if self.res.len() == 0 {
            return None;
        }
        Some(self.res.remove(0))
    }

    /// Returns the buffer's length that wait for decoding. It usually is `0`. Non-zero means that
    /// decoder need more buffer.
    pub fn buffer_len(&self) -> usize {
        self.buf.len()
    }

    /// Returns decoded massages count. The decoded messages will be hold by decoder,
    /// until you read them.
    pub fn result_len(&self) -> usize {
        self.res.len()
    }

    fn parse(&mut self) -> Result<usize> {
        let mut count: usize = 0;
        while self.msg_end <= self.buf.len() {
            if self.msg_start > self.pos {
                let len: usize = self.msg_end - self.msg_start;
                let mut buf: Vec<u8> = Vec::with_capacity(len);
                if len > 0 {
                    copy_to_vec(&self.buf, &mut buf, 0, self.msg_start, len);
                }
                count += 1;
                self.res.push(buf);
                self.pos = self.msg_end;
                self.msg_start = self.msg_end;
            }

            if self.buf.len() == self.pos {
                self.pos = 0;
                self.msg_start = 0;
                self.msg_end = 0;
                self.buf.clear();
                return Ok(count);
            }

            match parse_buffer(&self.buf, self.pos) {
                Ok((0, 0)) => {}
                Ok((start, end)) => {
                    self.msg_start = start;
                    self.msg_end = end;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(count)
    }
}

fn parse_buffer(buf: &[u8], pos: usize) -> Result<(usize, usize)> {
    let mut byte: u8 = 0;
    let mut total: usize = 0;
    let mut pos: usize = pos;
    loop {
        let len = unsafe { *buf.get_unchecked(pos) as usize };
        pos += 1;
        if len < 128 {
            return Ok((pos, pos + total + len));
        }
        byte += 1;
        if byte >= 4 {
            return Err(Error::new(ErrorKind::InvalidData,
                                  "Max buffer length must be small than 268435456"));
        }
        if pos >= buf.len() {
            return Ok((0, 0));
        }
        total = ((len & 0x7f) + total) * 128;
    }
}

fn copy_to_vec(src: &[u8], dst: &mut Vec<u8>, dst_start: usize, src_start: usize, count: usize) {
    let len = dst.len();
    unsafe {
        ptr::copy_nonoverlapping(src.get_unchecked(src_start),
                                 dst.get_unchecked_mut(dst_start),
                                 count);
        dst.set_len(len + count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_len_eq_0() {
        assert_eq!(encode(&vec![]), vec![0]);
    }

    #[test]
    fn encode_len_lt_128() {
        assert_eq!(encode(&vec![0x1u8, 0x2u8]), vec![0x2u8, 0x1u8, 0x2u8]);

        let mut res = Vec::with_capacity(127 + 1);
        res.extend_from_slice(&[0x7fu8]);
        res.append(&mut vec![0xffu8; 127]);
        assert_eq!(encode(&vec![0xffu8; 127]), res);
    }

    #[test]
    fn encode_len_eq_128() {
        let mut res = Vec::with_capacity(128 + 2);
        res.extend_from_slice(&[0x81u8, 0x00u8]);
        res.append(&mut vec![0xffu8; 128]);
        assert_eq!(encode(&vec![0xffu8; 128]), res);
    }

    #[test]
    fn encode_len_lt_16384() {
        let mut res = Vec::with_capacity(129 + 2);
        res.extend_from_slice(&[0x81u8, 0x01u8]);
        res.append(&mut vec![0xffu8; 129]);
        assert_eq!(encode(&vec![0xffu8; 129]), res);

        let mut res = Vec::with_capacity(16383 + 2);
        res.extend_from_slice(&[0xffu8, 0x7fu8]);
        res.append(&mut vec![0xffu8; 16383]);
        assert_eq!(encode(&vec![0xffu8; 16383]), res);
    }

    #[test]
    fn encode_len_eq_16384() {
        let mut res = Vec::with_capacity(16384 + 3);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x00u8]);
        res.append(&mut vec![0xffu8; 16384]);
        assert_eq!(encode(&vec![0xffu8; 16384]), res);
    }

    #[test]
    fn encode_len_lt_2097152() {
        let mut res = Vec::with_capacity(16385 + 3);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x01u8]);
        res.append(&mut vec![0xffu8; 16385]);
        assert_eq!(encode(&vec![0xffu8; 16385]), res);

        let mut res = Vec::with_capacity(2097151 + 3);
        res.extend_from_slice(&[0xffu8, 0xffu8, 0x7fu8]);
        res.append(&mut vec![0xffu8; 2097151]);
        assert_eq!(encode(&vec![0xffu8; 2097151]), res);
    }

    #[test]
    fn encode_len_eq_2097152() {
        let mut res = Vec::with_capacity(2097152 + 4);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x80u8, 0x00u8]);
        res.append(&mut vec![0xffu8; 2097152]);
        assert_eq!(encode(&vec![0xffu8; 2097152]), res);
    }

    #[test]
    fn encode_len_lt_268435456() {
        let mut res = Vec::with_capacity(2097153 + 4);
        res.extend_from_slice(&[0x81u8, 0x80u8, 0x80u8, 0x01u8]);
        res.append(&mut vec![0xffu8; 2097153]);
        assert_eq!(encode(&vec![0xffu8; 2097153]), res);

        let mut res = Vec::with_capacity(268435455 + 4);
        res.extend_from_slice(&[0xffu8, 0xffu8, 0xffu8, 0x7fu8]);
        res.append(&mut vec![0xffu8; 268435455]);
        assert_eq!(encode(&vec![0xffu8; 268435455]), res);
    }

    #[test]
    fn decode_zero_buf() {
        let res = decode((&vec![]));
        assert_eq!(res, None);
    }

    #[test]
    fn decode_half_prefix_buf() {
        let res = decode((&vec![0x81u8, 0x80u8]));
        assert_eq!(res, None);
    }

    #[test]
    fn decode_half_buf() {
        let res = decode((&vec![0x3u8, 0x01u8, 0x02u8]));
        assert_eq!(res, None);
    }

    #[test]
    fn decode_sim_buf() {
        let res = decode((&vec![0x3u8, 0x01u8, 0x02u8, 0x03u8]));
        assert_eq!(res.unwrap(), vec![0x01u8, 0x02u8, 0x03u8]);
    }

    #[test]
    fn decode_buf() {
        let res = decode(&encode(&mut vec![0xffu8; 16384]));
        assert_eq!(res.unwrap(), vec![0xffu8; 16384]);
    }

    #[test]
    fn decoder_sim() {
        let mut decoder = Decoder::new();
        decoder.feed(&encode(&vec![0xffu8; 16384])).unwrap();
        assert_eq!(decoder.read().unwrap(), vec![0xffu8; 16384]);
    }
}
