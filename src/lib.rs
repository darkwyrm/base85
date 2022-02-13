//! A library for Base85 encoding as described in [RFC1924](https://datatracker.ietf.org/doc/html/rfc1924) and released under the Mozilla Public License 2.0.
//!
//!## Description
//!
//! Several variants of Base85 encoding exist. The most popular variant is often know also as ascii85 and is best known for use in Adobe products. This is not that algorithm.
//!
//! The variant implemented in RFC 1924 was originally intended for encoding IPv6 addresses. It utilizes the same concepts as other versions, but uses a character set which is friendly toward embedding in source code without the need for escaping. During decoding ASCII whitespace (\n, \r, \t, space) is ignored. A base85-encoded string is 25% larger than the original binary data, which is more efficient than the more-common base64 algorithm (33%). This encoding pairs very well with JSON, yielding lower overhead and needing no character escapes.
//!
//! ## Usage
//!
//! Although this code is a work-in-progress and my first real Rust code, the API is simple: `encode()` turns a slice of bytes into a String and `decode()` turns a string reference into a Vector of bytes (u8). Both calls work completely within RAM, so processing huge files is probably not a good idea.
//!
//! ## Contributions
//!
//! I've been coding for a while, but I'm still a beginner at Rust. Suggestions and contributions are always welcome.

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

lazy_static! {
	static ref BASE85_CHARS: Vec<u8> =
		"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~"
			.bytes()
			.collect();
	static ref DECODEMAP: HashMap<u8, u8> = {
		let mut m = HashMap::new();
		let mut i: u8 = 0;
		for c in BASE85_CHARS.iter() {
			m.insert(*c, i.clone());
			i += 1;
		}

		m
	};
}

/// encode() turns a slice of bytes into a string of encoded data
pub fn encode(indata: &[u8]) -> String {
	if indata.len() == 0 {
		return String::from("")
	}

	let mut outdata: Vec<u8> = Vec::new();

	let length = indata.len();
	let chunk_count = (length / 4) as u32;
	let mut data_index: usize = 0;

	for _i in 0..chunk_count {
		let decnum: u32 = (indata[data_index] as u32).overflowing_shl(24).0
			| (indata[data_index + 1] as u32).overflowing_shl(16).0
			| (indata[data_index + 2] as u32).overflowing_shl(8).0
			| indata[data_index + 3] as u32;

		outdata.push(BASE85_CHARS[decnum as usize / 52200625]);
		let mut remainder = decnum as usize % 52200625;
		outdata.push(BASE85_CHARS[remainder / 614125]);

		remainder %= 614125;
		outdata.push(BASE85_CHARS[remainder / 7225]);

		remainder %= 7225;
		outdata.push(BASE85_CHARS[remainder / 85]);

		outdata.push(BASE85_CHARS[remainder % 85]);

		data_index += 4;
	}

	let extra_bytes = length % 4;
	if extra_bytes != 0 {
		let mut last_chunk = 0_u32;

		for i in length - extra_bytes..length {
			last_chunk = last_chunk.overflowing_shl(8).0;
			last_chunk |= indata[i] as u32;
		}

		// Pad extra bytes with zeroes
		{
			let mut i = 4 - extra_bytes;
			while i > 0 {
				last_chunk = last_chunk.overflowing_shl(8).0;
				i -= 1;
			}
		}

		outdata.push(BASE85_CHARS[last_chunk as usize / 52200625]);
		let mut remainder = last_chunk as usize % 52200625;
		outdata.push(BASE85_CHARS[remainder / 614125]);

		if extra_bytes > 1 {
			remainder %= 614125;
			outdata.push(BASE85_CHARS[remainder / 7225]);

			if extra_bytes > 2 {
				remainder %= 7225;
				outdata.push(BASE85_CHARS[remainder / 85]);
			}
		}
	}

	String::from_utf8(outdata).unwrap()
}

/// decode() turns a string of encoded data into a slice of bytes
pub fn decode(instr: &str) -> Option<Vec<u8>> {

	let length = instr.len() as u32;
	let mut outdata = Vec::<u8>::new();
	let mut accumulator;
	let mut in_index = instr.bytes();

	for _chunk in 0..length / 5 {
		accumulator = 0;

		// This construct is a bit strange because Rust doesn't let us modify the index variable
		// in a for loop
		{
			let mut i = 0;
			while i < 5 {
				let b = match in_index.next() {
					Some(n) => n,
					_ => break,
				};
				match b {
					32 | 10 | 11 | 13 => {
						// Ignore whitespace
						continue;
					}
					_ => {}
				}

				match DECODEMAP.get(&b) {
					Some(value) => {
						accumulator = (accumulator * 85) + *value as u32;
					},
					_ => return None,
				}
				i += 1;
			}
		}
		outdata.push((accumulator >> 24) as u8);
		outdata.push(((accumulator >> 16) & 255) as u8);
		outdata.push(((accumulator >> 8) & 255) as u8);
		outdata.push((accumulator & 255) as u8);
	}

	let remainder = length % 5;
	if remainder > 0 {
		accumulator = 0;
		{
			let mut i = 0;
			while i < 5 {
				let value: u8;

				if i < remainder {
					let b = match in_index.next() {
						Some(n) => n,
						_ => break,
					};
					match b {
						32 | 10 | 11 | 13 => {
							// Ignore whitespace
							continue;
						}
						_ => {}
					}

					value = match DECODEMAP.get(&b) {
						Some(x) => *x,
						_ => return None,
					}
				} else {
					value = 126;
				}
				accumulator = (accumulator * 85) + value as u32;
				i += 1;
			}
		}

		match remainder {
			4 => {
				outdata.push((accumulator >> 24) as u8);
				outdata.push(((accumulator >> 16) & 255) as u8);
				outdata.push(((accumulator >> 8) & 255) as u8);
			}
			3 => {
				outdata.push((accumulator >> 24) as u8);
				outdata.push(((accumulator >> 16) & 255) as u8);
			}
			2 => {
				outdata.push((accumulator >> 24) as u8);
			}
			_ => panic!(),
		}
	}

	Some(outdata)
}

#[cfg(test)]
#[test]
fn test_encode_decode() {

	// The list of tests consists of the unencoded data on the left and the encoded data on
	// the right. By using strings for the arbitrary binary data, we make the test much less
	// complicated to write.
	let testlist = [
		("a", "VE"),
		("aa", "VPO" ),
		("aaa", "VPRn" ),
		("aaaa", "VPRom" ),
		("aaaaa", "VPRomVE" ),
		("aaaaaa", "VPRomVPO" ),
		("aaaaaaa", "VPRomVPRn"),
		("aaaaaaaa", "VPRomVPRom")
	];

	for test in testlist.iter() {
		let s = encode(test.0.as_bytes());
		assert_eq!(s, test.1, "encoder test failed: wanted: {}, got: {}", test.0, s);

		let b = match decode(test.1) {
			Some(v) => v,
			_ => panic!("decoder test error on input {}", test.1)
		};

		let s = match String::from_utf8(b) {
			Ok(v) => v,
			Err(e) => panic!("decoder test '{}' failed to convert to string, error {}", test.1, e)
		};

		assert_eq!(test.0, s, "decoder data mismatch: wanted: {}, got: {}", test.0, s);
	}
}
