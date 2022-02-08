#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::error::Error;

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
pub fn decode(instr: &str) -> Result<Vec<u8>, Box<dyn Error>> {
	// TODO: Implement decode()

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
					}
					_ => return Err(format!("Bad value {} in data", b).into()),
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
						_ => return Err(format!("Bad value {} in data", b).into()),
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

	Ok(outdata)
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
		assert_eq!(s, test.1, format!("encoder test failed: wanted: {}, got: {}", test.0, s));

		let b = match decode(test.1) {
			Ok(v) => v,
			Err(e) => panic!(format!("decoder test failed on input {}", test.1))
		};

		let s = match String::from_utf8(b) {
			Ok(v) => v,
			Err(e) => panic!(format!("decoder test '{}' failed to convert to string", test.1))
		};

		assert_eq!(test.1, s, format!("decoder data mismatch: wanted: {}, got: {}", test.1, s));
	}
}
