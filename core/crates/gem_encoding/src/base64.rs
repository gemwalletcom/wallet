use crate::base_n::Alphabet;
use crate::{EncodingError, EncodingType};

static STANDARD: Alphabet<3, 4> = Alphabet::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/");
static URL_SAFE: Alphabet<3, 4> = Alphabet::new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_");

pub fn encode_base64(bytes: &[u8]) -> String {
    STANDARD.encode(bytes, true)
}

pub fn decode_base64(value: &str) -> Result<Vec<u8>, EncodingError> {
    let unpadded = value.trim_end_matches('=');
    if value.len() != unpadded.len().div_ceil(4) * 4 {
        return Err(EncodingError::Invalid(EncodingType::Base64, "invalid padding".to_string()));
    }
    STANDARD.decode(unpadded.as_bytes(), EncodingType::Base64)
}

pub fn decode_base64_no_pad(value: &str) -> Result<Vec<u8>, EncodingError> {
    STANDARD.decode(value.as_bytes(), EncodingType::Base64)
}

pub fn encode_base64_url(bytes: &[u8]) -> String {
    URL_SAFE.encode(bytes, false)
}

pub fn decode_base64_url(value: &str) -> Result<Vec<u8>, EncodingError> {
    URL_SAFE.decode(value.as_bytes(), EncodingType::Base64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_4648_VECTORS: [(&str, &str); 7] = [("", ""), ("f", "Zg=="), ("fo", "Zm8="), ("foo", "Zm9v"), ("foob", "Zm9vYg=="), ("fooba", "Zm9vYmE="), ("foobar", "Zm9vYmFy")];

    #[test]
    fn test_base64_rfc_4648_vectors() {
        for (plain, encoded) in RFC_4648_VECTORS {
            assert_eq!(encode_base64(plain.as_bytes()), encoded);
            assert_eq!(decode_base64(encoded).unwrap(), plain.as_bytes(), "{encoded}");
            let unpadded = encoded.trim_end_matches('=');
            assert_eq!(decode_base64_no_pad(unpadded).unwrap(), plain.as_bytes(), "{unpadded}");
            assert_eq!(encode_base64_url(plain.as_bytes()), unpadded);
            assert_eq!(decode_base64_url(unpadded).unwrap(), plain.as_bytes(), "{unpadded}");
        }
    }

    #[test]
    fn test_base64_round_trip_every_length() {
        for length in 0..=64usize {
            let bytes: Vec<u8> = (0..length).map(|index| (index * 37 + 251) as u8).collect();
            assert_eq!(decode_base64(&encode_base64(&bytes)).unwrap(), bytes);
            assert_eq!(decode_base64_url(&encode_base64_url(&bytes)).unwrap(), bytes);
            assert_eq!(decode_base64_no_pad(encode_base64(&bytes).trim_end_matches('=')).unwrap(), bytes);
        }
    }

    #[test]
    fn test_decode_base64_rejects_non_canonical_input() {
        for value in [
            "Zg",
            "Zm8",
            "Zm9vYg",
            "Zg=",
            "Zg===",
            "Zm8==",
            "Zm9v=",
            "Zm9v====",
            "=",
            "==",
            "====",
            "Zg==Zg==",
            "Zm8=Zm8=",
            "Zh==",
            "Zm9=",
            "Zm9v\n",
            " Zm9v",
            "Zm9v!",
            "-_8=",
            "Zm9vYg==\0",
        ] {
            assert!(decode_base64(value).is_err(), "{value:?}");
        }
    }

    #[test]
    fn test_decode_base64_no_pad_rejects_padding_and_bad_length() {
        for value in ["Zm9vYg==", "Zm8=", "Zm9vY", "Zh", "Zm9v Yg", "-_8"] {
            assert!(decode_base64_no_pad(value).is_err(), "{value:?}");
        }
    }

    #[test]
    fn test_decode_base64_url_uses_url_alphabet_without_padding() {
        assert_eq!(decode_base64_url("-_8").unwrap(), [0xfb, 0xff]);
        assert_eq!(encode_base64_url(&[0xfb, 0xff]), "-_8");
        for value in ["+/8", "-_8=", "-_9", "Zm9vY"] {
            assert!(decode_base64_url(value).is_err(), "{value:?}");
        }
    }
}
