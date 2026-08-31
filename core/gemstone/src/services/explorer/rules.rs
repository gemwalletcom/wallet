pub fn selected_explorer(explorers: &[String], selected: Option<String>) -> Option<String> {
    selected.filter(|name| explorers.contains(name)).or_else(|| explorers.first().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_explorer() {
        let explorers = vec!["Etherscan".to_string(), "Blockscout".to_string()];
        assert_eq!(selected_explorer(&explorers, Some("Blockscout".to_string())), Some("Blockscout".to_string()));
        assert_eq!(selected_explorer(&explorers, Some("Unknown".to_string())), Some("Etherscan".to_string()));
        assert_eq!(selected_explorer(&explorers, None), Some("Etherscan".to_string()));
        assert_eq!(selected_explorer(&[], None), None);
    }
}
