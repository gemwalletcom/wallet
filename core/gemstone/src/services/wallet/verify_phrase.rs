use std::fmt;

use super::rules::{PHRASE_VERIFICATION_GROUP, phrase_verification_words};
use crate::models::button::GemButtonState;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseSession {
    pub expected: Vec<u32>,
    pub chips: Vec<u32>,
    pub picked: Vec<u32>,
    pub is_creating: bool,
}

#[derive(Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseSetup {
    pub choices: Vec<String>,
    pub session: GemVerifyPhraseSession,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseChoice {
    pub index: u32,
    pub is_picked: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemVerifyPhraseViewState {
    pub verified_count: u32,
    pub groups: Vec<Vec<GemVerifyPhraseChoice>>,
    pub current_group: Option<u32>,
    pub next_index: Option<u32>,
    pub is_complete: bool,
    pub button: GemButtonState,
}

impl fmt::Debug for GemVerifyPhraseSetup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GemVerifyPhraseSetup").field("word_count", &self.choices.len()).field("session", &self.session).finish()
    }
}

impl GemVerifyPhraseSetup {
    pub fn new(words: Vec<String>) -> Self {
        let choices = phrase_verification_words(words.clone());
        Self::shuffled(&words, choices)
    }

    fn shuffled(words: &[String], choices: Vec<String>) -> Self {
        let identity = |word: &String| words.iter().position(|candidate| candidate == word).unwrap_or(words.len()) as u32;
        Self {
            session: GemVerifyPhraseSession {
                expected: words.iter().map(identity).collect(),
                chips: choices.iter().map(identity).collect(),
                picked: vec![],
                is_creating: false,
            },
            choices,
        }
    }
}

impl GemVerifyPhraseSession {
    fn accepts(&self, choice: u32) -> bool {
        let expected = self.expected.get(self.picked.len());
        let chip = self.chips.get(choice as usize);
        chip.is_some() && chip == expected && !self.picked.contains(&choice)
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
        let is_complete = !self.expected.is_empty() && verified_count == self.expected.len();
        let next_index = (verified_count < self.expected.len()).then_some(verified_count as u32);
        let choices: Vec<GemVerifyPhraseChoice> = (0..self.chips.len() as u32)
            .map(|index| GemVerifyPhraseChoice {
                index,
                is_picked: self.picked.contains(&index),
            })
            .collect();
        GemVerifyPhraseViewState {
            verified_count: verified_count as u32,
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

    fn setup(words: &[&str], choices: &[&str]) -> GemVerifyPhraseSetup {
        let words: Vec<String> = words.iter().map(|word| word.to_string()).collect();
        GemVerifyPhraseSetup::shuffled(&words, choices.iter().map(|word| word.to_string()).collect())
    }

    fn session(words: &[&str], choices: &[&str]) -> GemVerifyPhraseSession {
        setup(words, choices).session
    }

    #[test]
    fn test_picking_the_words_in_order_completes_the_phrase() {
        let session = session(&["alpha", "beta", "gamma"], &["gamma", "alpha", "beta"]);

        let state = session.on_pick(1).on_pick(2).view_state();
        assert_eq!(state.verified_count, 2);
        assert_eq!(state.next_index, Some(2));
        assert!(!state.is_complete);

        let state = session.on_pick(1).on_pick(2).on_pick(0).view_state();
        assert_eq!(state.verified_count, 3);
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
        let setup = GemVerifyPhraseSetup::new(words.clone());

        let state = setup.session.view_state();
        assert_eq!(state.groups.len(), 6);
        assert!(state.groups.iter().all(|group| group.len() == PHRASE_VERIFICATION_GROUP));
        assert_eq!(state.groups.iter().flatten().map(|choice| choice.index).collect::<Vec<_>>(), (0..24).collect::<Vec<u32>>());
        assert_eq!(state.current_group, Some(0));

        let picked = (0..5).fold(setup.session.clone(), |session, position| {
            let choice = setup.choices.iter().position(|word| *word == words[position]).unwrap() as u32;
            session.on_pick(choice)
        });
        assert_eq!(picked.view_state().current_group, Some(1));
    }

    #[test]
    fn test_the_session_carries_no_words_and_the_setup_never_prints_them() {
        let setup = setup(&["alpha", "beta"], &["beta", "alpha"]);
        let session = setup.session.on_pick(1);

        let printed = format!("{setup:?} {session:?} {:?}", session.view_state());
        assert!(!printed.contains("alpha") && !printed.contains("beta"), "{printed}");
    }

    #[test]
    fn test_a_new_setup_shuffles_the_phrase_into_its_choices() {
        let words: Vec<String> = (1..=12).map(|number| number.to_string()).collect();

        let setup = GemVerifyPhraseSetup::new(words.clone());

        let mut choices = setup.choices.clone();
        choices.sort();
        let mut expected = words;
        expected.sort();
        assert_eq!(choices, expected);
        assert!(setup.session.picked.is_empty());
        assert_eq!(setup.session.view_state().next_index, Some(0));
    }
}
