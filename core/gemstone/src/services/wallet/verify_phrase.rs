use std::fmt;

use super::rules::phrase_verification_words;

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseSession {
    pub words: Vec<String>,
    pub choices: Vec<String>,
    pub picked: Vec<u32>,
}

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseChoice {
    pub word: String,
    pub is_picked: bool,
}

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseViewState {
    pub verified: Vec<String>,
    pub choices: Vec<GemVerifyPhraseChoice>,
    pub next_index: Option<u32>,
    pub is_complete: bool,
}

impl fmt::Debug for GemVerifyPhraseSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseSession").field("word_count", &self.words.len()).field("picked", &self.picked).finish()
    }
}

impl fmt::Debug for GemVerifyPhraseChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseChoice").field("is_picked", &self.is_picked).finish()
    }
}

impl fmt::Debug for GemVerifyPhraseViewState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseViewState")
            .field("verified_count", &self.verified.iter().filter(|word| !word.is_empty()).count())
            .field("choices", &self.choices)
            .field("next_index", &self.next_index)
            .field("is_complete", &self.is_complete)
            .finish()
    }
}

impl GemVerifyPhraseSession {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            choices: phrase_verification_words(words.clone()),
            words,
            picked: vec![],
        }
    }

    fn accepts(&self, choice: u32) -> bool {
        let expected = self.words.get(self.picked.len());
        let word = self.choices.get(choice as usize);
        word.is_some() && word == expected && !self.picked.contains(&choice)
    }
}

#[uniffi::export]
impl GemVerifyPhraseSession {
    pub fn on_pick(&self, choice: u32) -> Self {
        if !self.accepts(choice) {
            return self.clone();
        }
        Self {
            picked: self.picked.iter().copied().chain([choice]).collect(),
            ..self.clone()
        }
    }

    pub fn view_state(&self) -> GemVerifyPhraseViewState {
        let verified_count = self.picked.len();
        GemVerifyPhraseViewState {
            verified: self.words.iter().enumerate().map(|(index, word)| if index < verified_count { word.clone() } else { String::new() }).collect(),
            choices: self
                .choices
                .iter()
                .enumerate()
                .map(|(index, word)| GemVerifyPhraseChoice {
                    word: word.clone(),
                    is_picked: self.picked.contains(&(index as u32)),
                })
                .collect(),
            next_index: (verified_count < self.words.len()).then_some(verified_count as u32),
            is_complete: !self.words.is_empty() && verified_count == self.words.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(words: &[&str], choices: &[&str]) -> GemVerifyPhraseSession {
        GemVerifyPhraseSession {
            words: words.iter().map(|word| word.to_string()).collect(),
            choices: choices.iter().map(|word| word.to_string()).collect(),
            picked: vec![],
        }
    }

    #[test]
    fn test_picking_the_words_in_order_completes_the_phrase() {
        let session = session(&["alpha", "beta", "gamma"], &["gamma", "alpha", "beta"]);

        let state = session.on_pick(1).on_pick(2).view_state();
        assert_eq!(state.verified, vec!["alpha", "beta", ""]);
        assert_eq!(state.next_index, Some(2));
        assert!(!state.is_complete);

        let state = session.on_pick(1).on_pick(2).on_pick(0).view_state();
        assert_eq!(state.verified, vec!["alpha", "beta", "gamma"]);
        assert_eq!(state.next_index, None);
        assert!(state.is_complete);
        assert!(state.choices.iter().all(|choice| choice.is_picked));
    }

    #[test]
    fn test_a_wrong_word_a_reused_chip_and_an_unknown_chip_change_nothing() {
        let session = session(&["alpha", "beta"], &["beta", "alpha"]);

        assert_eq!(session.on_pick(0), session, "beta is not the first word");
        assert_eq!(session.on_pick(7), session, "there is no eighth chip");
        let picked = session.on_pick(1);
        assert_eq!(picked.on_pick(1), picked, "a chip is used once");
        assert_eq!(picked.view_state().choices.iter().map(|choice| choice.is_picked).collect::<Vec<_>>(), vec![false, true]);
    }

    #[test]
    fn test_a_repeated_word_accepts_either_of_its_chips_for_either_place() {
        let session = session(&["echo", "echo", "fox"], &["echo", "fox", "echo"]);

        assert!(session.on_pick(2).on_pick(0).on_pick(1).view_state().is_complete);
        assert!(session.on_pick(0).on_pick(2).on_pick(1).view_state().is_complete);
    }

    #[test]
    fn test_debug_output_never_prints_the_phrase() {
        let session = session(&["alpha", "beta"], &["beta", "alpha"]).on_pick(1);

        let printed = format!("{session:?} {:?}", session.view_state());

        assert!(!printed.contains("alpha") && !printed.contains("beta"), "{printed}");
    }

    #[test]
    fn test_a_new_session_shuffles_the_phrase_into_its_choices() {
        let words: Vec<String> = (1..=12).map(|number| number.to_string()).collect();

        let session = GemVerifyPhraseSession::new(words.clone());

        let mut choices = session.choices.clone();
        choices.sort();
        let mut expected = words;
        expected.sort();
        assert_eq!(choices, expected);
        assert!(session.picked.is_empty());
        assert_eq!(session.view_state().next_index, Some(0));
    }
}
