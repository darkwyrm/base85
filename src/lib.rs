//! A library for Base85 encoding as described in [RFC1924](https://datatracker.ietf.org/doc/html/rfc1924) and released under the Mozilla Public License 2.0.
//!
//!## Description
//!
//! Several variants of Base85 encoding exist. The most popular variant is often known as ascii85 and is best known for use in Adobe products. This is not that algorithm.
//!
//! The variant implemented in RFC 1924 was originally intended for encoding IPv6 addresses. It utilizes the same concepts as other versions, but uses a character set which is friendly toward embedding in source code without the need for escaping. During decoding ASCII whitespace (\n, \r, \t, space) is ignored. A base85-encoded string is 25% larger than the original binary data, which is more efficient than the more-common base64 algorithm (33%). This encoding pairs very well with JSON, yielding lower overhead and needing no character escapes.
//!
//! ## Usage
//!
//! This was my first real Rust project but has matured since then and is stable. The API is simple: `encode()` turns a slice of bytes into a String and `decode()` turns a string reference into a Vector of bytes (u8). Both calls work completely within RAM, so processing huge files is probably not a good idea.
//!
//! ## Contributions
//!
//! Even though I've been coding for a while and have learned quite a bit about Rust, but I'm still a novice. Suggestions and contributions are always welcome and appreciated.

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected end of input")]
    UnexpectedEof,
    #[error("Unexpected character '{0}'")]
    InvalidCharacter(u8),
}

use itertools::Itertools;
use core::borrow::Borrow;

/// encode() turns a slice of bytes into a string of encoded data
pub fn encode(indata: impl IntoIterator<Item=impl Borrow<u8>>) -> String {
    #[inline]
    fn byte_to_char85(x85: u8) -> u8 {
        unsafe { *b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~".get_unchecked(x85 as usize) }
    }

    let mut v = Vec::<u8>::new();

    let mut id = indata.into_iter();
    loop {
        match (id.next().map(|x|*x.borrow()), id.next().map(|x|*x.borrow()), id.next().map(|x|*x.borrow()), id.next().map(|x|*x.borrow())) {
            (Some(a),Some(b),Some(c),Some(d)) => {
                let decnum = u32::from_be_bytes([a, b, c, d]);
                v.extend([
                    byte_to_char85((decnum / 85u32.pow(4)) as u8),
                    byte_to_char85(((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8),
                    byte_to_char85(((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8),
                    byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8),
                    byte_to_char85((decnum % 85u32) as u8)
                ]);
            },
            (Some(a),b,c,d) => {
                let decnum = u32::from_be_bytes([a, b.unwrap_or(0), c.unwrap_or(0), d.unwrap_or(0)]);
                v.push(byte_to_char85((decnum / 85u32.pow(4)) as u8));
                v.push(byte_to_char85(((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8));
                if b.is_some() {
                    v.push(byte_to_char85(((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8));
                }
                if c.is_some() {
                    v.push(byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8));
                }
                if d.is_some() {
                    v.push(byte_to_char85((decnum % 85u32) as u8));
                }
            },
            (None, _, _, _) => {
                break;
            }
        }
    }

    unsafe { String::from_utf8_unchecked(v) }
}

pub fn decode(indata: impl IntoIterator<Item=impl Borrow<u8>>) -> Result<Vec<u8>> {
    #[inline]
    fn char85_to_byte(c: u8) -> Result<u8> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'A'..=b'Z' => Ok(c - b'A' + 10),
            b'a'..=b'z' => Ok(c - b'a' + 36),
            b'!' => Ok(62),
            b'#' => Ok(63),
            b'$' => Ok(64),
            b'%' => Ok(65),
            b'&' => Ok(66),
            b'(' => Ok(67),
            b')' => Ok(68),
            b'*' => Ok(69),
            b'+' => Ok(70),
            b'-' => Ok(71),
            b';' => Ok(72),
            b'<' => Ok(73),
            b'=' => Ok(74),
            b'>' => Ok(75),
            b'?' => Ok(76),
            b'@' => Ok(77),
            b'^' => Ok(78),
            b'_' => Ok(79),
            b'`' => Ok(80),
            b'{' => Ok(81),
            b'|' => Ok(82),
            b'}' => Ok(83),
            b'~' => Ok(84),
            v => Err(Error::InvalidCharacter(v)),
        }
    }

    indata
        .into_iter()
        .map(|v|*v.borrow())
        .filter(|v| !(*v == 32 || *v == 10 || *v == 11 || *v == 13))
        .chunks(5)
        .into_iter()
        .map(|mut v| {
            let (a,b,c,d,e) = (v.next(), v.next(), v.next(), v.next(), v.next());
            let accumulator = u32::from(char85_to_byte(a.unwrap())?) * 85u32.pow(4)
                + u32::from(b.map_or(Err(Error::UnexpectedEof), char85_to_byte)?) * 85u32.pow(3)
                + u32::from(c.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(2)
                + u32::from(d.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(1)
                + u32::from(e.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(0);
            Ok([
                Some((accumulator >> 24) as u8),
                c.map(|_|(accumulator >> 16) as u8),
                d.map(|_|(accumulator >> 8) as u8),
                e.map(|_|accumulator as u8)
            ])
        })
        .flatten_ok()
        .filter_map_ok(|v| v)
        .collect::<Result<Vec<u8>>>()
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_encode_decode() {
        // The list of tests consists of the unencoded data on the left and the encoded data on
        // the right. By using strings for the arbitrary binary data, we make the test much less
        // complicated to write.
        let testlist = [
            ("a", "VE"),
            ("aa", "VPO"),
            ("aaa", "VPRn"),
            ("aaaa", "VPRom"),
            ("aaaaa", "VPRomVE"),
            ("aaaaaa", "VPRomVPO"),
            ("aaaaaaa", "VPRomVPRn"),
            ("aaaaaaaa", "VPRomVPRom"),
        ];

        for test in testlist.iter() {
            let s = encode(test.0.as_bytes());
            assert_eq!(
                s, test.1,
                "encoder test {} failed: wanted: {}, got: {}",
                test.0, test.1, s
            );

            let b = decode(test.1.bytes())
                .unwrap_or_else(|e| panic!("decoder test error on input {}: {}", test.1, e));

            let s = String::from_utf8(b).unwrap_or_else(|e| {
                panic!(
                    "decoder test '{}' failed to convert to string: {:#?}",
                    test.1, e
                )
            });

            assert_eq!(
                test.0, s,
                "decoder data mismatch: wanted: {}, got: {}",
                test.0, s
            );
        }
    }
}
