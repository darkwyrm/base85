//! this is no allocation api for base85 encoding and decoding. it allow to encode and decode in place, so it can be used in no_std environment without heap allocation. the input and output buffers must be large enough to hold the encoded or decoded data. the function will return the number of bytes written to the output buffer. the input buffer can be reused after encoding or decoding.
//!
//!
use crate::Error;
use crate::{byte_to_char85, char85_to_byte};
pub type Result<T> = std::result::Result<T, Error>;

/// During encoding, this function is used to calculate size to allocate for output buffer.
pub fn calc_encode_len(indata_bytes_len: usize) -> usize {
    let chunks_num = indata_bytes_len / 4;
    let remain = indata_bytes_len - (chunks_num * 4); // Modulo is more expensive than sub, so we do it this way
    if remain == 0 {
        chunks_num * 5
    } else {
        chunks_num * 5 + remain + 1
    }
}

/// During decoding, this function is used to calculate size to allocate for output buffer.
pub fn calc_decode_len(indata_bytes_len: usize) -> usize {
    let chunks_num = indata_bytes_len / 5;
    let remain = indata_bytes_len - (chunks_num * 5); // Mod is more expensive than sub, so we do it this way
    if remain == 0 {
        chunks_num * 4
    } else {
        chunks_num * 4 + remain - 1
    }
}

pub fn encode2(indata: &[u8]) -> String {
    let capacity = calc_encode_len(indata.len());
    let mut out = Vec::with_capacity(capacity); // No initialization of the buffer is needed, as encode_noalloc will write to the entire buffer. We can safely set the length to the capacity after encoding.
    unsafe {
        out.set_len(capacity);
    }
    let _ = match encode_noalloc(indata, &mut out) {
        Ok(encoded) => encoded,
        Err(e) => panic!("Error encoding test data: {e} at line: {}", line!()),
    };
    unsafe { String::from_utf8_unchecked(out) }
    // Encoding result is always a valid UTF-8 string, so we can safely use from_utf8_unchecked here. This is a micro optimization to avoid the overhead of checking for UTF-8 validity, which we know is guaranteed by the encoding process.
}

pub fn encode_noalloc<'a>(indata: &[u8], out: &'a mut [u8]) -> Result<&'a mut [u8]> {
    let chunks = indata.chunks_exact(4);
    let remainder = chunks.remainder();
    let final_encoded_len = calc_encode_len(indata.len());
    if out.len() < final_encoded_len {
        return Err(Error::OutputBufferTooSmall);
    }
    let out = &mut out[..final_encoded_len];
    let mut out_chunks = out.chunks_exact_mut(5);

    for (chunk, out) in std::iter::zip(chunks, &mut out_chunks) {
        let decnum = u32::from_be_bytes(<[u8; 4]>::try_from(chunk).unwrap());
        out[0] = byte_to_char85((decnum / 85u32.pow(4)) as u8);
        out[1] = byte_to_char85(((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8);
        out[2] = byte_to_char85(((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8);
        out[3] = byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8);
        out[4] = byte_to_char85((decnum % 85u32) as u8);
    }

    let out_remainder = out_chunks.into_remainder();
    if let Some(a) = remainder.first().copied() {
        let b = remainder.get(1).copied();
        let c = remainder.get(2).copied();
        let d = remainder.get(3).copied();
        let decnum = u32::from_be_bytes([a, b.unwrap_or(0), c.unwrap_or(0), d.unwrap_or(0)]);
        out_remainder[0] = byte_to_char85((decnum / 85u32.pow(4)) as u8);
        out_remainder[1] = byte_to_char85(((decnum % 85u32.pow(4)) / 85u32.pow(3)) as u8);
        if b.is_some() {
            out_remainder[2] = byte_to_char85(((decnum % 85u32.pow(3)) / 85u32.pow(2)) as u8);
        }
        if c.is_some() {
            out_remainder[3] = byte_to_char85(((decnum % 85u32.pow(2)) / 85u32) as u8);
        }
        if d.is_some() {
            out_remainder[4] = byte_to_char85((decnum % 85u32) as u8);
        }
    }

    Ok(&mut out[..final_encoded_len])
}

pub fn decode2(indata: &[u8]) -> Result<Vec<u8>> {
    let capacity = calc_decode_len(indata.len());
    let mut out = Vec::with_capacity(capacity); // No initialization of the buffer is needed, as decode_noalloc will write to the entire buffer. We can safely set the length to the capacity after encoding.
    unsafe {
        out.set_len(capacity);
    }
    let _ = decode_noalloc(indata, &mut out).map_err(|_e| Error::UnexpectedEof)?;
    Ok(out)
}

/// decode() turns a string of encoded data into a slice of bytes
pub fn decode_noalloc<'a>(indata: &[u8], out: &'a mut [u8]) -> Result<&'a mut [u8]> {
    let chunks = indata.chunks_exact(5);
    let remainder = chunks.remainder();
    let final_encoded_len = calc_decode_len(indata.len());
    if out.len() < final_encoded_len {
        return Err(Error::OutputBufferTooSmall);
    }
    let out = &mut out[..final_encoded_len];

    let mut out_chunks = out.chunks_exact_mut(4);

    for (chunk, out_chunk) in std::iter::zip(chunks, &mut out_chunks) {
        let accumulator = u32::from(char85_to_byte(chunk[0])?) * 85u32.pow(4)
            + u32::from(char85_to_byte(chunk[1])?) * 85u32.pow(3)
            + u32::from(char85_to_byte(chunk[2])?) * 85u32.pow(2)
            + u32::from(char85_to_byte(chunk[3])?) * 85u32
            + u32::from(char85_to_byte(chunk[4])?);
        out_chunk[0] = (accumulator >> 24) as u8;
        out_chunk[1] = (accumulator >> 16) as u8;
        out_chunk[2] = (accumulator >> 8) as u8;
        out_chunk[3] = accumulator as u8;
    }

    let out_remainder = out_chunks.into_remainder();
    if let Some(a) = remainder.first().copied() {
        let b = remainder.get(1).copied();
        let c = remainder.get(2).copied();
        let d = remainder.get(3).copied();
        let e = remainder.get(4).copied();
        let accumulator = u32::from(char85_to_byte(a)?) * 85u32.pow(4)
            + u32::from(b.map_or(Err(Error::UnexpectedEof), char85_to_byte)?) * 85u32.pow(3)
            + u32::from(c.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(2)
            + u32::from(d.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(1)
            + u32::from(e.map_or(Ok(126), char85_to_byte)?) * 85u32.pow(0);
        out_remainder[0] = (accumulator >> 24) as u8;
        if remainder.len() > 2 {
            out_remainder[1] = (accumulator >> 16) as u8;
            if remainder.len() > 3 {
                out_remainder[2] = (accumulator >> 8) as u8;
                if remainder.len() > 4 {
                    out_remainder[3] = accumulator as u8;
                }
            }
        }
    }

    Ok(&mut out[..final_encoded_len])
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    // Check with https://nerdmosis.com/tools/encode-and-decode-base85
    const RFC1924_ALPHABET_ENCODED :&str= "FflSSG&MFiI5|N=LqtVJM@UIZOH55pPf$@(Q&d$}S6EqEVPa!sWoBn+X=-b1ZEkOHadLBXb#`}nd3qruBqb&&DJm;1J3Ku;KR{kzV0(Oheg";
    const fn get_rfc1924_dic_as_str() -> &'static str {
        unsafe { std::str::from_utf8_unchecked(crate::RFC1924_ALPHABET) }
    }
    const TESTLIST: [(&str, &str); 9] = [
        ("a", "VE"),
        ("aa", "VPO"),
        ("aaa", "VPRn"),
        ("aaaa", "VPRom"),
        ("aaaaa", "VPRomVE"),
        ("aaaaaa", "VPRomVPO"),
        ("aaaaaaa", "VPRomVPRn"),
        ("aaaaaaaa", "VPRomVPRom"),
        (get_rfc1924_dic_as_str(), RFC1924_ALPHABET_ENCODED),
    ];

    #[test]
    fn test_encode_decode() -> Result<()> {
        // The list of tests consists of the decoded data on the left and the encoded data on
        // the right. By using strings for the arbitrary binary data, we make the test much less
        // complicated to write.

        let max_len = TESTLIST.iter().fold(0 as usize, |acc, test| {
            calc_encode_len(test.0.len()).max(acc)
        });
        assert_eq!(107, max_len, "test data is too long for the output buffer");
        let mut output_orig = vec![0u8; max_len];
        let mut output = &mut output_orig[..];
        for test in TESTLIST.iter() {
            let resu = encode_noalloc(test.0.as_bytes(), &mut output)?;

            assert_eq!(
                test.1.as_bytes(),
                resu,
                "encoder test failed: wanted: {:?}, got: {:?}",
                test.0.as_bytes(),
                resu
            );

            let b = decode_noalloc(test.1.as_bytes(), &mut output)
                .unwrap_or_else(|e| panic!("decoder test error on input {}: {}", test.1, e));

            let s = str::from_utf8(b).unwrap_or_else(|e| {
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
        Ok(())
    }

    #[test]
    fn test_check_len() -> Result<()> {
        for (input, expected_output) in TESTLIST.iter() {
            assert_eq!(input.len(), calc_decode_len(expected_output.len()));
            assert_eq!(expected_output.len(), calc_encode_len(input.len()));
        }
        Ok(())
    }

    #[test]
    fn unit_encode_all_possible_chars() -> Result<()> {
        let all_possible_encoded:&str="009C61O)~M2nh-c3=Iws5D^j+6crX17#SKH9337XAR!_nBqb&%C@Cr{EG;fCFflSSG&MFiI5|2yJUu=?KtV!7L`6nNNJ&adOifNtP*GA-R8>}2SXo+ITwPvYU}0ioWMyV&XlZI|Y;A6DaB*^Tbai%jczJqze0_d@fPsR8goTEOh>41ejE#<ukdcy;l$Dm3n3<ZJoSmMZprN9pq@|{(sHv)}tgWuEu(7hUw6(UkxVgH!yuH4^z`?@9#Kp$P$jQpf%+1cv(9zP<)YaD4*xB0K+}+;a;Njxq<mKk)=;`X~?CtLF@bU8V^!4`l`1$(#{Qds_";
        let mut input = Vec::<u8>::with_capacity(256);
        for i in 0..=255 {
            input.push(i as u8);
        }
        let encoded = encode2(&input);
        assert_eq!(all_possible_encoded, encoded);

        let decoded = decode2(encoded.as_bytes())?;
        assert_eq!(input, decoded);
        Ok(())
    }
}
