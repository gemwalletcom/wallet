use crate::base_n::Alphabet;
use crate::{EncodingError, EncodingType};

static STANDARD: Alphabet<5, 8> = Alphabet::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567");

pub fn encode_base32(bytes: &[u8]) -> String {
    STANDARD.encode(bytes, false)
}

pub fn decode_base32(value: &[u8]) -> Result<Vec<u8>, EncodingError> {
    STANDARD.decode(value, EncodingType::Base32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32_rfc_4648_vectors() {
        for (plain, encoded) in [("", ""), ("f", "MY"), ("fo", "MZXQ"), ("foo", "MZXW6"), ("foob", "MZXW6YQ"), ("fooba", "MZXW6YTB"), ("foobar", "MZXW6YTBOI")] {
            assert_eq!(encode_base32(plain.as_bytes()), encoded);
            assert_eq!(decode_base32(encoded.as_bytes()).unwrap(), plain.as_bytes(), "{encoded}");
        }
    }

    #[test]
    fn test_base32_round_trip_every_length() {
        for length in 0..=64usize {
            let bytes: Vec<u8> = (0..length).map(|index| (index * 37 + 251) as u8).collect();
            assert_eq!(decode_base32(encode_base32(&bytes).as_bytes()).unwrap(), bytes);
        }
    }

    #[test]
    fn test_decode_base32_rejects_non_canonical_input() {
        for value in ["M", "MZX", "MZXW6Y", "MZ", "MZXR", "MY======", "my", "MZXW6=", "MZXW1", "MZXW 6", "MZXW6\n", "\u{e9}Y"] {
            assert!(decode_base32(value.as_bytes()).is_err(), "{value:?}");
        }
    }
}
