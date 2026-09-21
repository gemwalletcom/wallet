pub const EMPTY_VALUE: &str = "-";

pub fn text_or_placeholder(value: Option<&str>) -> String {
    match value.map(str::trim) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => EMPTY_VALUE.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_blank_value_reads_as_a_dash() {
        assert_eq!(text_or_placeholder(Some("  ")), "-");
        assert_eq!(text_or_placeholder(None), "-");
        assert_eq!(text_or_placeholder(Some(" 1 ")), "1");
    }
}
