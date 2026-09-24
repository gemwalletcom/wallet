use std::fmt;

use super::rules::{PHRASE_VERIFICATION_GROUP, phrase_verification_words};
use crate::models::button::GemButtonState;

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseSession {
    pub words: Vec<String>,
    pub choices: Vec<String>,
    pub picked: Vec<u32>,
    pub is_creating: bool,
}

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseChoice {
    pub index: u32,
    pub word: String,
    pub is_picked: bool,
}

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseViewState {
    pub verified: Vec<String>,
    pub groups: Vec<Vec<GemVerifyPhraseChoice>>,
    pub current_group: Option<u32>,
    pub next_index: Option<u32>,
    pub is_complete: bool,
    pub button: GemButtonState,
}

impl fmt::Debug for GemVerifyPhraseSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseSession").field("word_count", &self.words.len()).field("picked", &self.picked).finish()
    }
}

impl fmt::Debug for GemVerifyPhraseChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseChoice").field("index", &self.index).field("is_picked", &self.is_picked).finish()
    }
}

impl fmt::Debug for GemVerifyPhraseViewState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseViewState")
            .field("verified_count", &self.verified.iter().filter(|word| !word.is_empty()).count())
            .field("groups", &self.groups)
            .field("current_group", &self.current_group)
            .field("next_index", &self.next_index)
            .field("is_complete", &self.is_complete)
            .field("button", &self.button)
            .finish()
    }
}

impl GemVerifyPhraseSession {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            choices: phrase_verification_words(words.clone()),
            words,
            picked: vec![],
            is_creating: false,
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

    pub fn on_creating(&self, is_creating: bool) -> Self {
        Self { is_creating, ..self.clone() }
    }

    pub fn view_state(&self) -> GemVerifyPhraseViewState {
        let verified_count = self.picked.len();
        let is_complete = !self.words.is_empty() && verified_count == self.words.len();
        let next_index = (verified_count < self.words.len()).then_some(verified_count as u32);
        let choices: Vec<GemVerifyPhraseChoice> = self
            .choices
            .iter()
            .enumerate()
            .map(|(index, word)| GemVerifyPhraseChoice {
                index: index as u32,
                word: word.clone(),
                is_picked: self.picked.contains(&(index as u32)),
            })
            .collect();
        GemVerifyPhraseViewState {
            verified: self.words.iter().enumerate().map(|(index, word)| if index < verified_count { word.clone() } else { String::new() }).collect(),
            groups: choices.chunks(PHRASE_VERIFICATION_GROUP).map(<[GemVerifyPhraseChoice]>::to_vec).collect(),
            current_group: next_index.map(|index| index / PHRASE_VERIFICATION_GROUP as u32),
            next_index,
            is_complete,
            button: match (self.is_creating, is_complete) {
                (true, _) => GemButtonState::Loading,
                (false, true) => GemButtonState::Enabled,
                (false, false) => GemButtonState::Disabled,
            },
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
            is_creating: false,
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
        assert!(state.groups.iter().flatten().all(|choice| choice.is_picked));
        assert_eq!(state.current_group, None);
    }

    #[test]
    fn test_the_button_opens_on_completion_and_loads_while_creating() {
        let session = session(&["alpha"], &["alpha"]);

        assert_eq!(session.view_state().button, GemButtonState::Disabled);
        assert_eq!(session.on_pick(0).view_state().button, GemButtonState::Enabled);
        assert_eq!(session.on_pick(0).on_creating(true).view_state().button, GemButtonState::Loading);
        assert_eq!(session.on_pick(0).on_creating(true).on_creating(false).view_state().button, GemButtonState::Enabled);
    }

    #[test]
    fn test_a_wrong_word_a_reused_chip_and_an_unknown_chip_change_nothing() {
        let session = session(&["alpha", "beta"], &["beta", "alpha"]);

        assert_eq!(session.on_pick(0), session, "beta is not the first word");
        assert_eq!(session.on_pick(7), session, "there is no eighth chip");
        let picked = session.on_pick(1);
        assert_eq!(picked.on_pick(1), picked, "a chip is used once");
        assert_eq!(picked.view_state().groups.iter().flatten().map(|choice| choice.is_picked).collect::<Vec<_>>(), vec![false, true]);
    }

    #[test]
    fn test_a_repeated_word_accepts_either_of_its_chips_for_either_place() {
        let session = session(&["echo", "echo", "fox"], &["echo", "fox", "echo"]);

        assert!(session.on_pick(2).on_pick(0).on_pick(1).view_state().is_complete);
        assert!(session.on_pick(0).on_pick(2).on_pick(1).view_state().is_complete);
    }

    #[test]
    fn test_choices_come_in_the_groups_they_were_shuffled_in_and_the_current_group_follows_the_next_word() {
        let words: Vec<String> = (0..24).map(|index| format!("word{index}")).collect();
        let session = GemVerifyPhraseSession::new(words.clone());

        let state = session.view_state();
        assert_eq!(state.groups.len(), 6);
        assert!(state.groups.iter().all(|group| group.len() == PHRASE_VERIFICATION_GROUP));
        assert_eq!(state.groups.iter().flatten().map(|choice| choice.index).collect::<Vec<_>>(), (0..24).collect::<Vec<u32>>());
        assert_eq!(state.current_group, Some(0));

        let picked = (0..5).fold(session.clone(), |session, position| {
            let choice = session.choices.iter().position(|word| *word == words[position]).unwrap() as u32;
            session.on_pick(choice)
        });
        assert_eq!(picked.view_state().current_group, Some(1));
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
