#![no_main]

use std::iter;

use gem_encoding::{Base58Alphabet, decode_base58, decode_base58_check, decode_base58_into, encode_base58, encode_base58_check};
use libfuzzer_sys::fuzz_target;

const SYMBOLS: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

fn reference(bytes: &[u8]) -> String {
    let mut digits = Vec::<u8>::new();
    for byte in bytes {
        let mut carry = u32::from(*byte);
        for digit in digits.iter_mut() {
            carry += u32::from(*digit) << 8;
            *digit = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let zeros = bytes.iter().take_while(|byte| **byte == 0).count();
    iter::repeat_n(SYMBOLS[0], zeros).chain(digits.iter().rev().map(|digit| SYMBOLS[usize::from(*digit)])).map(char::from).collect()
}

fuzz_target!(|input: (u8, &[u8])| {
    let (size, data) = input;
    let encoded = encode_base58(data);
    assert_eq!(encoded, reference(data));
    assert_eq!(decode_base58(&encoded).expect("an encoded value decodes"), data);
    if let Ok(bytes) = decode_base58(data) {
        assert_eq!(encode_base58(&bytes).as_bytes(), data);
        let mut output = vec![0u8; usize::from(size)];
        match decode_base58_into(data, &mut output) {
            Ok(length) => assert_eq!(output[..length], bytes),
            Err(_) => assert!(bytes.len() > output.len(), "a decoded value that fits the buffer decodes into it"),
        }
    }
    for alphabet in [Base58Alphabet::Bitcoin, Base58Alphabet::Ripple] {
        assert_eq!(decode_base58_check(&encode_base58_check(data, alphabet), alphabet).expect("an encoded value decodes"), data);
        if let Ok(text) = str::from_utf8(data)
            && let Ok(payload) = decode_base58_check(text, alphabet)
        {
            assert_eq!(encode_base58_check(&payload, alphabet).as_bytes(), data);
        }
    }
});
