pub fn is_name_supported(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2 && parts.last().is_some_and(|suffix| !suffix.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_name_supported() {
        assert!(is_name_supported("vitalik.eth"));
        assert!(is_name_supported("a.b.sol"));
        assert!(!is_name_supported("vitalik"));
        assert!(!is_name_supported("vitalik."));
        assert!(!is_name_supported("0x1234"));
    }
}
