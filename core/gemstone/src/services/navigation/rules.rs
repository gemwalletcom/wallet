pub fn code(code: Option<String>) -> Option<String> {
    code.map(|code| code.trim().to_string()).filter(|code| !code.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_rewards_link_with_no_code_opens_the_screen_without_one() {
        assert_eq!(code(Some("gemcoder".to_string())), Some("gemcoder".to_string()));
        assert_eq!(code(Some("  gemcoder ".to_string())), Some("gemcoder".to_string()));
        assert_eq!(code(Some(String::new())), None, "an empty code is no code on both apps");
        assert_eq!(code(Some("   ".to_string())), None);
        assert_eq!(code(None), None);
    }
}
