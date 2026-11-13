use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone)]
pub struct HexError(String);

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for HexError {}

pub fn encode(data: impl AsRef<[u8]>) -> String {
    gem_encoding::encode_hex(data)
}

pub fn decode(value: impl AsRef<[u8]>) -> Result<Vec<u8>, HexError> {
    gem_encoding::decode_hex(value).map_err(|error| HexError(error.to_string()))
}

pub fn encode_with_0x(data: &[u8]) -> String {
    format!("0x{}", encode(data))
}

pub fn parse_u64_from_hex_or_decimal(value: &str) -> Result<u64, HexError> {
    let number_text = value.trim();
    if let Some(hex_number_text) = number_text.strip_prefix("0x").or_else(|| number_text.strip_prefix("0X")) {
        return u64::from_str_radix(hex_number_text, 16).map_err(|error| HexError(error.to_string()));
    }
    number_text.parse::<u64>().map_err(|error| HexError(error.to_string()))
}

pub fn decode_hex_utf8(value: &str) -> Option<String> {
    let bytes = decode_hex(value).ok()?;
    String::from_utf8(bytes).ok()
}

pub fn decode_hex(value: &str) -> Result<Vec<u8>, HexError> {
    let stripped = value.trim().trim_prefix("0x");
    if stripped.is_empty() {
        return Ok(vec![]);
    }
    let normalized: Cow<str> = if stripped.len() % 2 == 1 { Cow::Owned(format!("0{stripped}")) } else { Cow::Borrowed(stripped) };
    decode(normalized.as_bytes())
}

pub fn decode_hex_array<const N: usize>(value: &str) -> Result<[u8; N], HexError> {
    let bytes = decode_hex(value)?;
    let length = bytes.len();
    bytes.try_into().map_err(|_| HexError(format!("hex value must be {N} bytes, got {length}")))
}

pub const fn decode_hex_literal<const N: usize>(value: &str) -> [u8; N] {
    let digits = value.as_bytes();
    assert!(digits.len() == N * 2, "hex literal needs two digits per byte");
    let mut bytes = [0u8; N];
    let mut index = 0;
    while index < N {
        bytes[index] = hex_digit(digits[index * 2]) << 4 | hex_digit(digits[index * 2 + 1]);
        index += 1;
    }
    bytes
}

const fn hex_digit(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => panic!("invalid hex digit"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROGRAM: [u8; 4] = decode_hex_literal("deadBEEF");

    #[test]
    fn test_decode_hex_literal() {
        assert_eq!(PROGRAM, [0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(decode_hex_literal::<0>(""), [0u8; 0]);
        assert_eq!(decode_hex_literal::<4>("00ff7f80"), [0x00, 0xff, 0x7f, 0x80]);
        assert_eq!(decode_hex_literal::<11>("0123456789abcdefABCDEF"), [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xab, 0xcd, 0xef]);
        assert_eq!(decode_hex_literal::<32>(&"ff".repeat(32)), [0xff; 32]);
    }

    #[test]
    #[should_panic(expected = "invalid hex digit")]
    fn test_decode_hex_literal_rejects_invalid_digit() {
        decode_hex_literal::<1>("0g");
    }

    #[test]
    #[should_panic(expected = "invalid hex digit")]
    fn test_decode_hex_literal_rejects_prefix() {
        decode_hex_literal::<2>("0x0a");
    }

    #[test]
    #[should_panic(expected = "hex literal needs two digits per byte")]
    fn test_decode_hex_literal_rejects_odd_length() {
        decode_hex_literal::<2>("abc");
    }

    #[test]
    #[should_panic(expected = "hex literal needs two digits per byte")]
    fn test_decode_hex_literal_rejects_length_mismatch() {
        decode_hex_literal::<4>("abcd");
    }

    #[test]
    fn decode_hex_trims_and_strips_prefix() {
        let bytes = decode_hex(" 0x0a0b ").expect("decode");
        assert_eq!(bytes, vec![0x0a, 0x0b]);
    }

    #[test]
    fn decode_hex_pads_odd_length() {
        let bytes = decode_hex("0xa").expect("decode");
        assert_eq!(bytes, vec![0x0a]);
    }

    #[test]
    fn decode_hex_array_validates_length() {
        assert_eq!(decode_hex_array::<2>("0x0a0b").expect("decode"), [0x0a, 0x0b]);
        assert_eq!(decode_hex_array::<2>("a0b").expect("decode"), [0x0a, 0x0b]);
        assert!(decode_hex_array::<2>("0x0a").is_err());
    }

    #[test]
    fn test_encode() {
        assert_eq!(encode([0x0a, 0x0b]), "0a0b");
        assert_eq!(encode_with_0x(&[0x0a, 0x0b]), "0x0a0b");
    }

    #[test]
    fn test_parse_u64_from_hex_or_decimal() {
        assert_eq!(parse_u64_from_hex_or_decimal("0x1f").unwrap(), 31);
        assert_eq!(parse_u64_from_hex_or_decimal("0X2A").unwrap(), 42);
        assert_eq!(parse_u64_from_hex_or_decimal("255").unwrap(), 255);
        assert!(parse_u64_from_hex_or_decimal("0xZZ").is_err());
    }
}
