pub fn is_valid_percent_encoding(value: &str) -> bool {
    value
        .split('%')
        .skip(1)
        .all(|part| part.as_bytes().get(..2).is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit)))
}

#[cfg(test)]
mod tests {
    use super::is_valid_percent_encoding;

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
