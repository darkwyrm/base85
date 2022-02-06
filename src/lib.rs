#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

const BASE85_CHARS: &str =
	"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";

lazy_static! {
	static ref DECODEMAP: HashMap<String, u8> = {
		let mut m = HashMap::new();
		let mut i: u8 = 0;
		for c in BASE85_CHARS.chars() {
			m.insert(c.to_string(), i.clone());
			i += 1;
		}

		m
	};
}

/// encode() turns a slice of bytes into a string of encoded data
pub fn encode(indata: &[u8]) -> String {
	// TODO: Implement encode()

	// let mut outdata: Vec<String> = Vec::new();
	// let mut out = String::new();

	// let length = indata.len();
	// let chunk_count = (length / 4) as usize;
	// let mut data_index: usize = 0;

	// for i in 0..chunk_count {
	// 	let decnum = indata.get(data_index);
	// }

	// out
	String::from("unimplementd")
}

/// decode() turns a string of encoded data into a slice of bytes
pub fn decode<'a>(instr: &'a str) -> Result<Vec<u8>, &'a str> {
	// TODO: Implement decode()
	Err("unimplemented")
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_encode() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}
