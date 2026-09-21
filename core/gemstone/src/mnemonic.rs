use std::fmt;

use gem_keystore::Mnemonic;

const PHRASE_SUGGESTION_LIMIT: usize = 20;

#[derive(Debug, Default, uniffi::Object)]
pub struct GemMnemonic;

#[derive(Clone, PartialEq)]
pub struct GemPhraseEdit {
    pub text: String,
    pub cursor: u32,
}

impl fmt::Debug for GemPhraseEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemPhraseEdit").field("cursor", &self.cursor).finish_non_exhaustive()
    }
}

#[uniffi::export]
impl GemMnemonic {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn find_invalid_words(&self, words: Vec<String>) -> Vec<String> {
        Mnemonic::invalid_words(&words.join(" "))
    }
}

pub(crate) fn phrase_suggestions(text: &str, cursor: u32) -> Vec<String> {
    let word = PhraseWord::at(text, cursor);
    let current = text[word.start..word.end].to_lowercase();
    Mnemonic::suggest_limited(&text[word.start..word.cursor], None)
        .into_iter()
        .filter(|suggestion| *suggestion != current)
        .take(PHRASE_SUGGESTION_LIMIT)
        .collect()
}

pub(crate) fn apply_phrase_suggestion(text: &str, cursor: u32, word: &str) -> GemPhraseEdit {
    let at = PhraseWord::at(text, cursor);
    let rest = &text[at.end..];
    let (separator, rest) = match rest.chars().next() {
        Some(next) if next.is_whitespace() => (&rest[..next.len_utf8()], &rest[next.len_utf8()..]),
        _ => (" ", rest),
    };
    let head = format!("{}{word}{separator}", &text[..at.start]);
    GemPhraseEdit {
        cursor: head.encode_utf16().count() as u32,
        text: head + rest,
    }
}

struct PhraseWord {
    start: usize,
    cursor: usize,
    end: usize,
}

impl PhraseWord {
    fn at(text: &str, cursor: u32) -> Self {
        let cursor = byte_offset(text, cursor as usize);
        let start = text[..cursor]
            .char_indices()
            .rev()
            .find(|(_, character)| character.is_whitespace())
            .map_or(0, |(index, character)| index + character.len_utf8());
        let end = text[cursor..].find(char::is_whitespace).map_or(text.len(), |index| cursor + index);
        Self { start, cursor, end }
    }
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

    fn suggestions(text: &str, cursor: usize) -> Vec<String> {
        phrase_suggestions(text, cursor as u32)
    }

    fn phrase_edit(text: &str, cursor: usize, word: &str) -> GemPhraseEdit {
        apply_phrase_suggestion(text, cursor as u32, word)
    }

    #[test]
    fn test_suggestions_follow_the_word_at_the_cursor() {
        assert_eq!(suggestions("woo", 3), vec!["wood", "wool"]);
        assert_eq!(suggestions("abandon woo zoo", 11), vec!["wood", "wool"], "the word being edited, not the last one");
        assert_eq!(suggestions("woo", 2)[..2], ["wolf", "woman"], "the typed part before the cursor is the prefix");
        assert!(suggestions("woof", 4).is_empty());
        assert!(suggestions("abandon ", 8).is_empty(), "nothing to complete after a space");
        assert!(suggestions("", 0).is_empty());
    }

    #[test]
    fn test_a_complete_word_is_not_offered_again() {
        assert_eq!(suggestions("act", 3), vec!["action", "actor", "actress", "actual"]);
        assert!(suggestions("wood", 4).is_empty());
        assert!(!suggestions("act", 3).contains(&"act".to_string()));
        assert!(!suggestions("action", 3).contains(&"action".to_string()), "the word already in place is dropped when editing inside it");
    }

    #[test]
    fn test_applying_a_suggestion_replaces_the_word_at_the_cursor() {
        assert_eq!(
            phrase_edit("abandon woo", 11, "wood"),
            GemPhraseEdit {
                text: "abandon wood ".to_string(),
                cursor: 13
            }
        );
        assert_eq!(phrase_edit("woo zoo", 3, "wood"), GemPhraseEdit { text: "wood zoo".to_string(), cursor: 5 });
        assert_eq!(
            phrase_edit("abandon wo zoo", 9, "wool"),
            GemPhraseEdit {
                text: "abandon wool zoo".to_string(),
                cursor: 13
            }
        );
        assert_eq!(phrase_edit("wo", 1, "wood"), GemPhraseEdit { text: "wood ".to_string(), cursor: 5 });
    }

    #[test]
    fn test_the_cursor_counts_utf16_units() {
        assert_eq!(suggestions("🙂 woo", 6), vec!["wood", "wool"]);
        assert_eq!(phrase_edit("🙂 woo", 6, "wood"), GemPhraseEdit { text: "🙂 wood ".to_string(), cursor: 8 });
        assert!(suggestions("woo", 99).len() == 2, "a cursor past the end reads as the end");
    }

    #[test]
    fn test_validate() {
        let mnemonic = GemMnemonic::new();
        let words = primitives::testkit::ABANDON_PHRASE.split_whitespace().map(|word| word.to_string()).collect::<Vec<_>>();

        assert!(mnemonic.find_invalid_words(words).is_empty());
        assert_eq!(mnemonic.find_invalid_words(vec!["abandon".to_string(), "test1".to_string()]), vec!["test1"]);
    }
}
