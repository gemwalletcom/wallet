use bip39::{Language, Mnemonic as Bip39Mnemonic};
use zeroize::Zeroizing;

use crate::KeystoreError;

pub type MnemonicSeed = [u8; 64];

pub struct Mnemonic;

impl Mnemonic {
    pub const MIN_ENTROPY_LEN: usize = 16;
    pub const MAX_ENTROPY_LEN: usize = 32;

    pub fn generate(word_count: usize) -> Result<Zeroizing<Vec<String>>, KeystoreError> {
        let entropy_len = entropy_len_for_word_count(word_count)?;
        let entropy = Zeroizing::new(gem_crypto::random::bytes::<32>()?);
        let mnemonic = Bip39Mnemonic::from_entropy_in(Language::English, &entropy[..entropy_len]).map_err(|_| KeystoreError::invalid_input("mnemonic"))?;
        Ok(Zeroizing::new(mnemonic.words().map(ToString::to_string).collect()))
    }

    pub fn clean(phrase: &str) -> Result<Zeroizing<String>, KeystoreError> {
        Self::parse(phrase).map(|(cleaned, _)| cleaned)
    }

    pub fn is_valid(phrase: &str) -> bool {
        Self::clean(phrase).is_ok()
    }

    pub fn is_valid_word(word: &str) -> bool {
        let word = word.trim().to_lowercase();
        !word.is_empty() && Language::English.find_word(&word).is_some()
    }

    pub fn invalid_words(phrase: &str) -> Vec<String> {
        phrase.split_whitespace().filter(|word| !Self::is_valid_word(word)).map(ToString::to_string).collect()
    }

    pub fn seed(phrase: &str) -> Result<Zeroizing<MnemonicSeed>, KeystoreError> {
        let (_, mnemonic) = Self::parse(phrase)?;
        Ok(Zeroizing::new(mnemonic.to_seed_normalized("")))
    }

    pub fn is_valid_entropy_len(len: usize) -> bool {
        (Self::MIN_ENTROPY_LEN..=Self::MAX_ENTROPY_LEN).contains(&len) && len.is_multiple_of(4)
    }

    pub fn entropy(phrase: &str) -> Result<Zeroizing<Vec<u8>>, KeystoreError> {
        let (_, mnemonic) = Self::parse(phrase)?;
        let (entropy, len) = mnemonic.to_entropy_array();
        Ok(Zeroizing::new(entropy[..len].to_vec()))
    }

    pub fn suggest_limited(prefix: &str, limit: Option<usize>) -> Vec<String> {
        let prefix = &prefix.trim().to_lowercase();
        if prefix.is_empty() {
            return Vec::new();
        }
        let words = Language::English.words_by_prefix_iter(prefix).map(str::to_string);
        match limit {
            Some(limit) => words.take(limit).collect(),
            None => words.collect(),
        }
    }

    fn parse(phrase: &str) -> Result<(Zeroizing<String>, Bip39Mnemonic), KeystoreError> {
        let words = Zeroizing::new(phrase.split_whitespace().map(str::to_lowercase).collect::<Vec<_>>());
        let cleaned = Zeroizing::new(words.join(" "));
        let mnemonic = Bip39Mnemonic::parse_in_normalized(Language::English, &cleaned).map_err(|_| KeystoreError::invalid_input("mnemonic"))?;
        Ok((cleaned, mnemonic))
    }
}

fn entropy_len_for_word_count(word_count: usize) -> Result<usize, KeystoreError> {
    match word_count {
        12 => Ok(Mnemonic::MIN_ENTROPY_LEN),
        15 => Ok(20),
        18 => Ok(24),
        21 => Ok(28),
        24 => Ok(Mnemonic::MAX_ENTROPY_LEN),
        _ => Err(KeystoreError::invalid_input("mnemonic")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recording_allocator::RecordingAllocator;
    use crate::testkit::ABANDON_PHRASE;

    #[test]
    fn test_phrase_words_are_wiped_before_free() {
        let freed_with_word = RecordingAllocator::count_freed(
            |layout, bytes| layout.align() == 1 && bytes.windows(b"abandon".len()).any(|window| window == b"abandon"),
            || {
                drop(Mnemonic::clean(ABANDON_PHRASE).unwrap());
                drop(Mnemonic::seed(ABANDON_PHRASE).unwrap());
                drop(Mnemonic::entropy(ABANDON_PHRASE).unwrap());
            },
        );

        assert_eq!(freed_with_word, 0);
    }

    #[test]
    fn test_suggest() {
        assert_eq!(Mnemonic::suggest_limited("woo", None), vec!["wood", "wool"]);
        assert_eq!(Mnemonic::suggest_limited("woo", Some(1)), vec!["wood"]);
        assert_eq!(Mnemonic::suggest_limited("abandon", None), vec!["abandon"]);
        assert_eq!(Mnemonic::suggest_limited("woof", None), Vec::<String>::new());

        let all_words = Mnemonic::suggest_limited(" ", None);
        assert_eq!(all_words.len(), 0);
    }

    #[test]
    fn test_entropy_len_for_word_count() {
        assert_eq!(entropy_len_for_word_count(12).unwrap(), Mnemonic::MIN_ENTROPY_LEN);
        assert_eq!(entropy_len_for_word_count(24).unwrap(), Mnemonic::MAX_ENTROPY_LEN);
        assert_eq!(entropy_len_for_word_count(13).unwrap_err(), KeystoreError::invalid_input("mnemonic"));

        for len in [16, 20, 24, 28, 32] {
            assert!(Mnemonic::is_valid_entropy_len(len));
        }
        for len in [0, 15, 17, 33] {
            assert!(!Mnemonic::is_valid_entropy_len(len));
        }
    }

    #[test]
    fn test_clean() {
        let phrase = format!("  {} ", ABANDON_PHRASE.replacen("abandon abandon", "ABANDON   abandon", 1).replace("about", "ABOUT"));
        assert_eq!(Mnemonic::clean(&phrase).unwrap().as_str(), ABANDON_PHRASE);
        assert_eq!(Mnemonic::clean("abandon abandon").unwrap_err(), KeystoreError::invalid_input("mnemonic"));
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
    fn test_clean_accepts_unicode_separators() {
        for separator in ["\t", "\n", "\r\n", "\u{a0}", "\u{2009}", "\u{3000}"] {
            let phrase = ABANDON_PHRASE.replace(' ', separator);
            assert_eq!(Mnemonic::clean(&phrase).unwrap().as_str(), ABANDON_PHRASE, "{separator:?}");
        }
    }

    #[test]
    fn test_compatibility_characters_are_invalid_words() {
        let full_width_abandon = "\u{ff41}\u{ff42}\u{ff41}\u{ff4e}\u{ff44}\u{ff4f}\u{ff4e}";
        assert!(!Mnemonic::is_valid_word(full_width_abandon));
        assert!(!Mnemonic::is_valid_word("\u{fb01}nal"));
        assert!(Mnemonic::clean(&ABANDON_PHRASE.replacen("abandon", full_width_abandon, 1)).is_err());
        assert_eq!(Mnemonic::invalid_words(&format!("abandon {full_width_abandon}")), vec![full_width_abandon]);
    }

    #[test]
    fn test_seed() {
        let seed = Mnemonic::seed(ABANDON_PHRASE).unwrap();
        assert_eq!(
            gem_encoding::encode_hex(seed.as_slice()),
            "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4"
        );
    }

    #[test]
    fn test_entropy() {
        let entropy = Mnemonic::entropy(ABANDON_PHRASE).unwrap();
        assert_eq!(entropy.len(), Mnemonic::MIN_ENTROPY_LEN);
        assert_eq!(gem_encoding::encode_hex(entropy.as_slice()), "00000000000000000000000000000000");
    }
}
