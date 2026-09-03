use gem_keystore::Mnemonic;

#[derive(Debug, Default, uniffi::Object)]
pub struct GemMnemonic;

#[uniffi::export]
impl GemMnemonic {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn suggest_words(&self, prefix: String, limit: Option<u32>) -> Vec<String> {
        Mnemonic::suggest_limited(&prefix, limit.map(|limit| limit as usize))
    }

    pub fn find_invalid_words(&self, words: Vec<String>) -> Vec<String> {
        Mnemonic::invalid_words(&words.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_words() {
        let mnemonic = GemMnemonic::new();

        assert_eq!(mnemonic.suggest_words("woo".to_string(), None), vec!["wood", "wool"]);
        assert_eq!(mnemonic.suggest_words("woo".to_string(), Some(1)), vec!["wood"]);
        assert_eq!(mnemonic.suggest_words("woof".to_string(), None), Vec::<String>::new());
    }

    #[test]
    fn test_validate() {
        let mnemonic = GemMnemonic::new();
        let words = primitives::testkit::ABANDON_PHRASE.split_whitespace().map(|word| word.to_string()).collect::<Vec<_>>();

        assert!(mnemonic.find_invalid_words(words).is_empty());
        assert_eq!(mnemonic.find_invalid_words(vec!["abandon".to_string(), "test1".to_string()]), vec!["test1"]);
    }
}
