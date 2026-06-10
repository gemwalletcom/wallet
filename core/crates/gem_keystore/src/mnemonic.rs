use bip39::{Language, Mnemonic as Bip39Mnemonic};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::KeystoreError;

pub struct Mnemonic;

impl Mnemonic {
    pub fn generate(word_count: usize) -> Result<Vec<String>, KeystoreError> {
        let entropy_len = entropy_len_for_word_count(word_count)?;
        let entropy = Zeroizing::new(gem_crypto::random::bytes::<32>()?);
        let mnemonic = Bip39Mnemonic::from_entropy_in(Language::English, &entropy[..entropy_len]).map_err(|_| KeystoreError::invalid_input("mnemonic"))?;
        Ok(mnemonic.words().map(|word| word.to_string()).collect())
    }

    pub fn sanitize(phrase: &str) -> Result<String, KeystoreError> {
        let sanitized = phrase.nfkd().collect::<String>();
        let sanitized = sanitized.split_whitespace().map(|word| word.to_lowercase()).collect::<Vec<_>>().join(" ");
        let mnemonic = Bip39Mnemonic::parse_in_normalized(Language::English, &sanitized).map_err(|_| KeystoreError::invalid_input("mnemonic"))?;
        Ok(mnemonic.words().collect::<Vec<_>>().join(" "))
    }

    pub fn is_valid(phrase: &str) -> bool {
        Self::sanitize(phrase).is_ok()
    }

    pub fn is_valid_word(word: &str) -> bool {
        let word = word.nfkd().collect::<String>().trim().to_lowercase();
        !word.is_empty() && Language::English.find_word(&word).is_some()
    }

    pub fn invalid_words(phrase: &str) -> Vec<String> {
        phrase.split_whitespace().filter(|word| !Self::is_valid_word(word)).map(|word| word.to_string()).collect()
    }

    pub fn seed(phrase: &str) -> Result<Zeroizing<[u8; 64]>, KeystoreError> {
        let sanitized = Zeroizing::new(Self::sanitize(phrase)?);
        let mnemonic = Bip39Mnemonic::parse_in_normalized(Language::English, &sanitized).map_err(|_| KeystoreError::invalid_input("mnemonic"))?;
        Ok(Zeroizing::new(mnemonic.to_seed_normalized("")))
    }

    pub fn entropy(phrase: &str) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        let sanitized = Zeroizing::new(Self::sanitize(phrase)?);
        let mnemonic = Bip39Mnemonic::parse_in_normalized(Language::English, &sanitized).map_err(|_| KeystoreError::invalid_input("mnemonic"))?;
        let (entropy, len) = mnemonic.to_entropy_array();
        Ok(Zeroizing::new(entropy[..len].to_vec()))
    }

    pub fn suggest(prefix: &str) -> Vec<String> {
        Self::suggest_limited(prefix, None)
    }

    pub fn suggest_limited(prefix: &str, limit: Option<usize>) -> Vec<String> {
        let prefix = &prefix.trim().to_lowercase();
        if prefix.is_empty() {
            return Vec::new();
        }
        let words = Language::English.words_by_prefix(prefix).iter().map(|word| word.to_string());
        match limit {
            Some(limit) => words.take(limit).collect(),
            None => words.collect(),
        }
    }
}

fn entropy_len_for_word_count(word_count: usize) -> Result<usize, KeystoreError> {
    match word_count {
        12 => Ok(16),
        15 => Ok(20),
        18 => Ok(24),
        21 => Ok(28),
        24 => Ok(32),
        _ => Err(KeystoreError::invalid_input("mnemonic")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::ABANDON_PHRASE;

    #[test]
    fn test_suggest() {
        assert_eq!(Mnemonic::suggest("woo"), vec!["wood", "wool"]);
        assert_eq!(Mnemonic::suggest_limited("woo", Some(1)), vec!["wood"]);
        assert_eq!(Mnemonic::suggest("abandon"), vec!["abandon"]);
        assert_eq!(Mnemonic::suggest("woof"), Vec::<String>::new());

        let all_words = Mnemonic::suggest(" ");
        assert_eq!(all_words.len(), 0);
    }

    #[test]
    fn test_entropy_len_for_word_count() {
        assert_eq!(entropy_len_for_word_count(12).unwrap(), 16);
        assert_eq!(entropy_len_for_word_count(24).unwrap(), 32);
        assert_eq!(entropy_len_for_word_count(13).unwrap_err(), KeystoreError::invalid_input("mnemonic"));
    }

    #[test]
    fn test_sanitize() {
        let phrase = format!("  {} ", ABANDON_PHRASE.replacen("abandon abandon", "ABANDON   abandon", 1).replace("about", "ABOUT"));
        assert_eq!(Mnemonic::sanitize(&phrase).unwrap(), ABANDON_PHRASE);
        assert_eq!(Mnemonic::sanitize("abandon abandon").unwrap_err(), KeystoreError::invalid_input("mnemonic"));
    }

    #[test]
    fn test_validate_words() {
        assert!(Mnemonic::is_valid(ABANDON_PHRASE));
        assert!(Mnemonic::is_valid(&ABANDON_PHRASE.to_uppercase()));
        assert!(!Mnemonic::is_valid("abandon abandon abandon"));

        assert!(Mnemonic::is_valid_word("abandon"));
        assert!(Mnemonic::is_valid_word(" ABANDON "));
        assert!(!Mnemonic::is_valid_word("abandon1"));
        assert_eq!(Mnemonic::invalid_words("abandon test1 about nope"), vec!["test1", "nope"]);
    }

    #[test]
    fn test_seed() {
        let seed = Mnemonic::seed(ABANDON_PHRASE).unwrap();
        assert_eq!(
            hex::encode(seed.as_slice()),
            "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4"
        );
    }

    #[test]
    fn test_entropy() {
        let entropy = Mnemonic::entropy(ABANDON_PHRASE).unwrap();
        assert_eq!(hex::encode(entropy.as_slice()), "00000000000000000000000000000000");
    }
}
