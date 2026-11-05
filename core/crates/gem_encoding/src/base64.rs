use crate::{EncodingError, EncodingType};
use data_encoding::{BASE64, BASE64_NOPAD, BASE64URL_NOPAD, Encoding};

pub fn encode_base64(bytes: &[u8]) -> String {
    BASE64.encode(bytes)
}

pub fn decode_base64(value: &str) -> Result<Vec<u8>, EncodingError> {
    if value.trim_end_matches('=').contains('=') {
        return Err(EncodingError::Invalid(EncodingType::Base64, "padding before the end".to_string()));
    }
    decode(&BASE64, value)
}

pub fn decode_base64_no_pad(value: &str) -> Result<Vec<u8>, EncodingError> {
    decode(&BASE64_NOPAD, value)
}

pub fn encode_base64_url(bytes: &[u8]) -> String {
    BASE64URL_NOPAD.encode(bytes)
}

pub fn decode_base64_url(value: &str) -> Result<Vec<u8>, EncodingError> {
    decode(&BASE64URL_NOPAD, value)
}

fn decode(encoding: &Encoding, value: &str) -> Result<Vec<u8>, EncodingError> {
    encoding.decode(value.as_bytes()).map_err(|error| EncodingError::Invalid(EncodingType::Base64, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_base64_requires_canonical_padding() {
        assert_eq!(decode_base64("Zm9vYg==").unwrap(), b"foob");
        assert!(decode_base64("Zm9vYg").is_err());
        assert!(decode_base64("Zm9vYh==").is_err());
        assert!(decode_base64("Zg==Zg==").is_err());
        assert_eq!(decode_base64_no_pad("Zm9vYg").unwrap(), b"foob");
        assert!(decode_base64_no_pad("Zm9vYg==").is_err());
        assert_eq!(decode_base64_url("-_8").unwrap(), [0xfb, 0xff]);
        assert!(decode_base64_url("+/8").is_err());
    }
}
