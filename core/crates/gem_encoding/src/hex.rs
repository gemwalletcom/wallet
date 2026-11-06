use crate::{EncodingError, EncodingType};

const DIGITS: &[u8; 16] = b"0123456789abcdef";

pub fn encode_hex(bytes: impl AsRef<[u8]>) -> String {
    let bytes = bytes.as_ref();
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

pub fn decode_hex(value: impl AsRef<[u8]>) -> Result<Vec<u8>, EncodingError> {
    let value = value.as_ref();
    if !value.len().is_multiple_of(2) {
        return Err(EncodingError::Invalid(EncodingType::Hex, "odd number of digits".to_string()));
    }
    value
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[high, low]| Some(digit(*high)? << 4 | digit(*low)?))
        .collect::<Option<Vec<u8>>>()
        .ok_or_else(|| EncodingError::Invalid(EncodingType::Hex, "invalid digit".to_string()))
}

fn digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_hex() {
        assert_eq!(encode_hex([]), "");
        assert_eq!(encode_hex([0x00, 0x0f, 0xf0, 0xff, 0x12, 0xab]), "000ff0ff12ab");
        assert_eq!(encode_hex("hi"), "6869");
        assert_eq!(encode_hex(vec![0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn test_decode_hex() {
        assert_eq!(decode_hex("").unwrap(), Vec::<u8>::new());
        assert_eq!(decode_hex("000ff0ff12ab").unwrap(), [0x00, 0x0f, 0xf0, 0xff, 0x12, 0xab]);
        assert_eq!(decode_hex("DeadBEEF").unwrap(), [0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(decode_hex(b"6869").unwrap(), b"hi");
        for invalid in ["0", "abc", "0x00", " 00", "00 ", "0g", "g0", "\u{e9}\u{e9}", "00\n"] {
            assert!(decode_hex(invalid).is_err(), "{invalid:?}");
        }
    }
}
