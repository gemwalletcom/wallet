use crate::services::transactions::GemAmountSign;
use primitives::PriceChangeCalculator as Calculator;

#[derive(Debug, uniffi::Object)]
pub struct PriceChangeCalculator {}

impl Default for PriceChangeCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl PriceChangeCalculator {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn pnl_percentage(&self, pnl: f64, margin: f64) -> f64 {
        Calculator::pnl_percentage(pnl, margin)
    }

    pub fn pnl_text(&self, formatted_amount: String, formatted_percentage: Option<String>) -> String {
        match formatted_percentage {
            Some(percentage) => format!("{formatted_amount} ({percentage})"),
            None => formatted_amount,
        }
    }

    pub fn sign(&self, value: f64) -> GemAmountSign {
        match value < 0.0 {
            true => GemAmountSign::Outgoing,
            false => GemAmountSign::Incoming,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GemAmountSign, PriceChangeCalculator};

    #[test]
    fn test_pnl_text_keeps_the_percentage_in_brackets_and_drops_it_when_there_is_none() {
        let calculator = PriceChangeCalculator::new();
        assert_eq!(calculator.pnl_text("+$500.00".to_string(), Some("+50.00%".to_string())), "+$500.00 (+50.00%)");
        assert_eq!(calculator.pnl_text("+$500.00".to_string(), None), "+$500.00");
    }

    #[test]
    fn test_a_flat_change_reads_as_a_gain() {
        let calculator = PriceChangeCalculator::new();
        assert_eq!(calculator.sign(1.5), GemAmountSign::Incoming);
        assert_eq!(calculator.sign(0.0), GemAmountSign::Incoming);
        assert_eq!(calculator.sign(-1.5), GemAmountSign::Outgoing);
    }
}
