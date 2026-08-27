pub fn parse_timestamp(value: Option<String>) -> Option<i64> {
    value.and_then(|value| value.trim().parse().ok())
}

pub fn parse_timestamp_or_zero(value: Option<String>) -> u64 {
    parse_timestamp(value).and_then(|timestamp| u64::try_from(timestamp).ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        assert_eq!(parse_timestamp(Some("42".to_string())), Some(42));
        assert_eq!(parse_timestamp(Some(" 7 ".to_string())), Some(7));
        assert_eq!(parse_timestamp(Some("abc".to_string())), None);
        assert_eq!(parse_timestamp(None), None);
        assert_eq!(parse_timestamp_or_zero(Some("42".to_string())), 42);
        assert_eq!(parse_timestamp_or_zero(Some("-1".to_string())), 0);
        assert_eq!(parse_timestamp_or_zero(None), 0);
    }
}
