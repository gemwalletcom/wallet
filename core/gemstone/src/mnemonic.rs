use gem_keystore::Mnemonic;

const PHRASE_SUGGESTION_LIMIT: usize = 20;

#[uniffi::export]
pub fn phrase_suggestions(word: String) -> Vec<String> {
    let current = word.to_lowercase();
    Mnemonic::suggest_limited(&word, None).into_iter().filter(|suggestion| *suggestion != current).take(PHRASE_SUGGESTION_LIMIT).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestions(word: &str) -> Vec<String> {
        phrase_suggestions(word.to_string())
    }

    #[test]
    fn test_suggestions_complete_the_word() {
        assert_eq!(suggestions("woo"), vec!["wood", "wool"]);
        assert!(suggestions("woof").is_empty());
        assert!(suggestions("").is_empty(), "nothing to complete after a space");
    }

    #[test]
    fn test_a_complete_word_is_not_offered_again() {
        assert_eq!(suggestions("act"), vec!["action", "actor", "actress", "actual"]);
        assert!(suggestions("wood").is_empty());
    }
}
