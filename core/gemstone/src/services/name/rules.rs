pub fn can_resolve_name(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() >= 2 && parts.last().is_some_and(|suffix| !suffix.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_resolve_name() {
        assert!(can_resolve_name("vitalik.eth"));
        assert!(can_resolve_name("a.b.sol"));
        assert!(!can_resolve_name("vitalik"));
        assert!(!can_resolve_name("vitalik."));
        assert!(!can_resolve_name("0x1234"));
    }
}
