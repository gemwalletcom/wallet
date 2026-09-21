use super::rules;
use crate::config::swap_config::get_swap_config;
use crate::models::swap::GemSlippageCheck;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSlippageSelection {
    Auto,
    Manual { bps: u32 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSlippageViewState {
    pub is_auto: bool,
    pub check: GemSlippageCheck,
    pub allows_confirm: bool,
    pub shows_warning: bool,
    pub selection: GemSlippageSelection,
    pub suggestions_bps: Vec<u32>,
    pub minimum_bps: u32,
    pub maximum_bps: u32,
    pub maximum_fraction_digits: u32,
    pub maximum_integer_digits: u32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSlippageSession {
    pub is_auto: bool,
    pub bps: u32,
}

impl GemSlippageSession {
    pub fn new(selection: GemSlippageSelection) -> Self {
        match selection {
            GemSlippageSelection::Auto => Self { is_auto: true, bps: 0 },
            GemSlippageSelection::Manual { bps } => Self { is_auto: false, bps },
        }
    }
}

#[uniffi::export]
impl GemSlippageSession {
    pub fn view_state(&self) -> GemSlippageViewState {
        let config = get_swap_config();
        let check = rules::slippage_check(self.bps, &config);
        GemSlippageViewState {
            is_auto: self.is_auto,
            check,
            allows_confirm: self.is_auto || check.allows_confirm(),
            shows_warning: !self.is_auto && check == GemSlippageCheck::High,
            selection: match self.is_auto {
                true => GemSlippageSelection::Auto,
                false => GemSlippageSelection::Manual { bps: self.bps },
            },
            suggestions_bps: config.slippage_suggestions_bps.clone(),
            minimum_bps: config.min_slippage_bps,
            maximum_bps: config.max_slippage_bps,
            maximum_fraction_digits: SLIPPAGE_FRACTION_DIGITS,
            maximum_integer_digits: rules::slippage_percent(config.max_slippage_bps).trunc().to_string().len() as u32,
        }
    }
}

const SLIPPAGE_FRACTION_DIGITS: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_confirms_whatever_the_input_says() {
        let above_maximum = GemSlippageSession::new(GemSlippageSelection::Manual { bps: get_swap_config().max_slippage_bps + 1 });

        assert!(!above_maximum.view_state().allows_confirm);
        let auto = GemSlippageSession::new(GemSlippageSelection::Auto);
        assert!(auto.view_state().allows_confirm, "auto does not read the input");
        assert!(!auto.view_state().shows_warning);
    }

    #[test]
    fn test_a_high_slippage_warns_but_still_confirms() {
        let state = GemSlippageSession::new(GemSlippageSelection::Manual {
            bps: get_swap_config().high_slippage_warning_bps,
        })
        .view_state();

        assert_eq!(state.check, GemSlippageCheck::High);
        assert!(state.shows_warning);
        assert!(state.allows_confirm);
    }

    #[test]
    fn test_the_integer_digits_follow_the_configured_maximum() {
        let state = GemSlippageSession::new(GemSlippageSelection::Manual { bps: 100 }).view_state();
        let maximum_percent = rules::slippage_percent(state.maximum_bps);

        assert_eq!(state.maximum_integer_digits, maximum_percent.trunc().to_string().len() as u32);
        assert_eq!(state.maximum_fraction_digits, 2);
    }
}
