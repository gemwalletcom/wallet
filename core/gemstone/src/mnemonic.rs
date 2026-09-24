use gem_keystore::Mnemonic;

const PHRASE_SUGGESTION_LIMIT: usize = 20;

#[uniffi::export]
pub fn phrase_suggestions(word: String, cursor: u32) -> Vec<String> {
    let prefix = &word[..byte_offset(&word, cursor as usize)];
    let current = word.to_lowercase();
    Mnemonic::suggest_limited(prefix, None).into_iter().filter(|suggestion| *suggestion != current).take(PHRASE_SUGGESTION_LIMIT).collect()
}

fn byte_offset(text: &str, utf16_offset: usize) -> usize {
    let mut units = 0;
    for (index, character) in text.char_indices() {
        if units >= utf16_offset {
            return index;
        }
        units += character.len_utf16();
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn suggestions(word: &str, cursor: usize) -> Vec<String> {
        phrase_suggestions(word.to_string(), cursor as u32)
    }

    #[test]
    fn test_suggestions_complete_the_typed_part_of_the_word() {
        assert_eq!(suggestions("woo", 3), vec!["wood", "wool"]);
        assert_eq!(suggestions("woo", 2)[..2], ["wolf", "woman"], "the typed part before the cursor is the prefix");
        assert!(suggestions("woof", 4).is_empty());
        assert!(suggestions("", 0).is_empty(), "nothing to complete between words");
    }

    #[test]
    fn test_a_complete_word_is_not_offered_again() {
        assert_eq!(suggestions("act", 3), vec!["action", "actor", "actress", "actual"]);
        assert!(suggestions("wood", 4).is_empty());
        assert!(!suggestions("act", 3).contains(&"act".to_string()));
        assert!(!suggestions("action", 3).contains(&"action".to_string()), "the word already in place is dropped when editing inside it");
    }

    #[test]
    fn test_the_cursor_counts_utf16_units() {
        assert!(suggestions("🙂", 2).is_empty());
        assert_eq!(suggestions("woo", 99), vec!["wood", "wool"], "a cursor past the end reads as the end");
    }
}
