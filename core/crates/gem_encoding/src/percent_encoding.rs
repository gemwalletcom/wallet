const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

pub fn is_valid_percent_encoding(value: &str) -> bool {
    value.split('%').skip(1).all(|part| part.as_bytes().get(..2).is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit)))
}

pub fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len() * 3);
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

pub fn percent_decode_lossy(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && let (Some(high), Some(low)) = (hex_digit(bytes.get(index + 1)), hex_digit(bytes.get(index + 2)))
        {
            decoded.push(high << 4 | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_digit(byte: Option<&u8>) -> Option<u8> {
    char::from(*byte?).to_digit(16).map(|digit| digit as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode() {
        assert_eq!(percent_encode(""), "");
        assert_eq!(percent_encode("AZaz09"), "AZaz09");
        assert_eq!(percent_encode("photo_01.png"), "photo%5F01%2Epng");
        assert_eq!(percent_encode("a b/c?d=e&f"), "a%20b%2Fc%3Fd%3De%26f");
        assert_eq!(percent_encode("-_.~%"), "%2D%5F%2E%7E%25");
        assert_eq!(percent_encode("é"), "%C3%A9");
        assert_eq!(percent_encode("\u{1F600}"), "%F0%9F%98%80");
    }

    #[test]
    fn test_percent_decode_lossy() {
        assert_eq!(percent_decode_lossy(""), "");
        assert_eq!(percent_decode_lossy("100%"), "100%");
        assert_eq!(percent_decode_lossy("%2541"), "%41");
        assert_eq!(percent_decode_lossy(&percent_encode("file name ü.png")), "file name ü.png");
        assert_eq!(percent_decode_lossy("Available%20amount%3A%200"), "Available amount: 0");
        assert_eq!(percent_decode_lossy("%c3%A9"), "é");
        assert_eq!(percent_decode_lossy("trailing%2"), "trailing%2");
        assert_eq!(percent_decode_lossy("%%41%GG%"), "%A%GG%");
        assert_eq!(percent_decode_lossy("%FF"), "\u{fffd}");
    }

    #[test]
    fn test_is_valid_percent_encoding() {
        for (value, expected) in [
            ("", true),
            ("plain", true),
            ("hello%20world", true),
            ("%00%ff%FF", true),
            ("%2520", true),
            ("%", false),
            ("%2", false),
            ("%GG", false),
            ("%2G", false),
            ("%%20", false),
            ("%20%", false),
            ("%é", false),
        ] {
            assert_eq!(is_valid_percent_encoding(value), expected, "{value}");
        }
    }
}
