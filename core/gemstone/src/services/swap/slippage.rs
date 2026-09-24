use primitives::Chain;

use super::rules;
use crate::config::swap_config::{get_default_slippage, get_swap_config};
use crate::formatted_number::GemFormattedNumber;
use crate::models::swap::GemSlippageCheck;
use crate::percentage::GemPercentageStyle;
use crate::services::amount::model::GemNumberFormat;
use crate::services::amount::rules::{plain_number, sanitize_number_input};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSlippageSelection {
    Auto,
    Manual { bps: u32 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSlippageSuggestion {
    pub bps: u32,
    pub percent: GemFormattedNumber,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSlippageViewState {
    pub is_auto: bool,
    pub input: String,
    pub placeholder: String,
    pub check: GemSlippageCheck,
    pub shows_check: bool,
    pub allows_confirm: bool,
    pub shows_warning: bool,
    pub selection: GemSlippageSelection,
    pub suggestions: Vec<GemSlippageSuggestion>,
    pub minimum: GemFormattedNumber,
    pub maximum: GemFormattedNumber,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSlippageSession {
    pub is_auto: bool,
    pub input: String,
    pub placeholder: String,
    pub format: GemNumberFormat,
}

#[uniffi::export]
pub fn new_slippage_session(selection: GemSlippageSelection, chain: Chain, format: GemNumberFormat) -> GemSlippageSession {
    let placeholder = rules::slippage_percent_text(get_default_slippage(&chain).bps, &format.decimal_separator);
    let (is_auto, input) = match selection {
        GemSlippageSelection::Auto => (true, String::new()),
        GemSlippageSelection::Manual { bps } => (false, rules::slippage_percent_text(bps, &format.decimal_separator)),
    };
    GemSlippageSession { is_auto, input, placeholder, format }
}

impl GemSlippageSession {
    fn bps(&self) -> Option<u32> {
        plain_number(&self.format.decimal_separator, &self.input).parse::<f64>().ok().and_then(rules::slippage_bps_from_percent)
    }
}

#[uniffi::export]
impl GemSlippageSession {
    pub fn on_auto(&self, is_auto: bool) -> Self {
        Self { is_auto, ..self.clone() }
    }

    pub fn on_input(&self, text: String) -> Self {
        let maximum_integer_digits = rules::slippage_percent(get_swap_config().max_slippage_bps).trunc().to_string().len() as u32;
        Self {
            input: sanitize_number_input(&self.format.decimal_separator, &text, Some(SLIPPAGE_FRACTION_DIGITS), Some(maximum_integer_digits)),
            ..self.clone()
        }
    }

    pub fn view_state(&self) -> GemSlippageViewState {
        let config = get_swap_config();
        let bps = self.bps();
        let check = rules::slippage_check(bps.unwrap_or(0), &config);
        GemSlippageViewState {
            is_auto: self.is_auto,
            input: self.input.clone(),
            placeholder: self.placeholder.clone(),
            check,
            shows_check: !self.is_auto && bps.is_some(),
            allows_confirm: self.is_auto || check.allows_confirm(),
            shows_warning: !self.is_auto && check == GemSlippageCheck::High,
            selection: match self.is_auto {
                true => GemSlippageSelection::Auto,
                false => GemSlippageSelection::Manual { bps: bps.unwrap_or(0) },
            },
            suggestions: config
                .slippage_suggestions_bps
                .iter()
                .map(|bps| GemSlippageSuggestion {
                    bps: *bps,
                    percent: percent(*bps),
                    input: rules::slippage_percent_text(*bps, &self.format.decimal_separator),
                })
                .collect(),
            minimum: percent(config.min_slippage_bps),
            maximum: percent(config.max_slippage_bps),
        }
    }
}

fn percent(bps: u32) -> GemFormattedNumber {
    GemFormattedNumber::percentage(rules::slippage_percent(bps), GemPercentageStyle::UnsignedCompact)
}

const SLIPPAGE_FRACTION_DIGITS: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    fn session(selection: GemSlippageSelection) -> GemSlippageSession {
        new_slippage_session(selection, Chain::Ethereum, GemNumberFormat { decimal_separator: ".".to_string() })
    }

    #[test]
    fn test_a_slippage_suggestion_carries_a_percent_no_app_has_to_sign() {
        let state = session(GemSlippageSelection::Auto).view_state();

        assert_eq!(state.suggestions.iter().map(|suggestion| suggestion.bps).collect::<Vec<_>>(), get_swap_config().slippage_suggestions_bps);
        assert!(
            state.suggestions.iter().all(|suggestion| suggestion.percent.unit == crate::formatted_number::GemNumberUnit::Percent),
            "a percent the apps append a literal sign to reads wrong in fr and tr"
        );
        assert_eq!(state.suggestions.iter().map(|suggestion| suggestion.input.as_str()).collect::<Vec<_>>(), vec!["0.3", "0.5", "3"]);
        assert_eq!(state.minimum.value, rules::slippage_percent(get_swap_config().min_slippage_bps));
        assert_eq!(state.maximum.value, rules::slippage_percent(get_swap_config().max_slippage_bps));
    }

    #[test]
    fn test_a_session_opens_on_the_selection_with_the_chain_default_as_placeholder() {
        let auto = session(GemSlippageSelection::Auto).view_state();
        assert!(auto.is_auto);
        assert_eq!(auto.input, "");
        assert_eq!(auto.placeholder, rules::slippage_percent_text(get_default_slippage(&Chain::Ethereum).bps, "."));

        let manual = new_slippage_session(GemSlippageSelection::Manual { bps: 50 }, Chain::Ethereum, GemNumberFormat { decimal_separator: ",".to_string() }).view_state();
        assert!(!manual.is_auto);
        assert_eq!(manual.input, "0,5");
        assert_eq!(manual.selection, GemSlippageSelection::Manual { bps: 50 });
    }

    #[test]
    fn test_typed_input_is_sanitized_and_read_as_the_selection() {
        let typed = session(GemSlippageSelection::Auto).on_auto(false).on_input("0.111111".to_string());
        assert_eq!(typed.input, "0.11");
        assert_eq!(typed.view_state().selection, GemSlippageSelection::Manual { bps: 11 });

        assert_eq!(session(GemSlippageSelection::Auto).on_input("33333312312".to_string()).input, "33");
    }

    #[test]
    fn test_an_incomplete_input_blocks_confirm_without_showing_the_check() {
        for text in ["", "0", "0.", "abc"] {
            let state = session(GemSlippageSelection::Manual { bps: 100 }).on_input(text.to_string()).view_state();
            assert!(!state.shows_check, "{text}");
            assert!(!state.allows_confirm, "{text}");
        }
        for text in ["25", "0.05"] {
            let state = session(GemSlippageSelection::Manual { bps: 100 }).on_input(text.to_string()).view_state();
            assert!(state.shows_check, "{text}");
            assert!(!state.allows_confirm, "{text}");
        }
    }

    #[test]
    fn test_auto_confirms_whatever_the_input_says() {
        let above_maximum = session(GemSlippageSelection::Manual {
            bps: get_swap_config().max_slippage_bps + 100,
        });

        assert!(!above_maximum.view_state().allows_confirm);
        let auto = above_maximum.on_auto(true).view_state();
        assert!(auto.allows_confirm, "auto does not read the input");
        assert!(!auto.shows_warning);
        assert!(!auto.shows_check);
        assert_eq!(auto.selection, GemSlippageSelection::Auto);
    }

    #[test]
    fn test_a_high_slippage_warns_but_still_confirms() {
        let state = session(GemSlippageSelection::Manual {
            bps: get_swap_config().high_slippage_warning_bps,
        })
        .view_state();

        assert_eq!(state.check, GemSlippageCheck::High);
        assert!(state.shows_warning);
        assert!(state.allows_confirm);
    }
}
