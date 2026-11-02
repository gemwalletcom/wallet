fn is_direction_character(character: char) -> bool {
    matches!(character, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

/// A name or message a third party writes shows on review screens, so it stays on one line and in reading order.
pub fn single_line(text: &str) -> String {
    let readable: String = text.chars().filter(|character| character.is_whitespace() || !(character.is_control() || is_direction_character(*character))).collect();
    readable.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line() {
        assert_eq!(single_line("Annual\n\n\nplan\u{202e}\u{0} for \u{2066}you\u{2069}"), "Annual plan for you");
        assert_eq!(single_line("Constant\u{202e} K"), "Constant K");
        assert_eq!(single_line("  Compound  "), "Compound");
        assert_eq!(single_line("\u{202e}\n"), "");
    }
}
