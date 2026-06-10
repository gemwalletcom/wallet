use gem_keystore::Mnemonic;

use crate::GemstoneError;

#[derive(Debug, Default, uniffi::Object)]
pub struct GemMnemonic;

#[uniffi::export]
impl GemMnemonic {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, word_count: u8) -> Result<Vec<String>, GemstoneError> {
        Ok(Mnemonic::generate(usize::from(word_count))?)
    }

    pub fn suggest_words(&self, prefix: String, limit: Option<u32>) -> Vec<String> {
        Mnemonic::suggest_limited(&prefix, limit.map(|limit| limit as usize))
    }

    pub fn is_valid_word(&self, word: String) -> bool {
        Mnemonic::is_valid_word(&word)
    }

    pub fn is_valid(&self, words: Vec<String>) -> bool {
        Mnemonic::is_valid(&words.join(" "))
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
    fn test_generate() {
        let mnemonic = GemMnemonic::new();

        assert_eq!(mnemonic.generate(12).unwrap().len(), 12);
    }

    #[test]
    fn test_validate() {
        let mnemonic = GemMnemonic::new();
        let words = primitives::testkit::ABANDON_PHRASE.split_whitespace().map(|word| word.to_string()).collect::<Vec<_>>();

        assert!(mnemonic.is_valid_word("abandon".to_string()));
        assert!(mnemonic.is_valid(words.clone()));
        assert!(!mnemonic.is_valid(vec!["abandon".to_string(), "abandon".to_string()]));
        assert_eq!(mnemonic.find_invalid_words(vec!["abandon".to_string(), "test1".to_string()]), vec!["test1"]);
    }
}
