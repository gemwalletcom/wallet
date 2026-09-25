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

    pub fn pnl_text(&self, formatted_amount: String, formatted_percentage: Option<String>) -> String {
        match formatted_percentage {
            Some(percentage) => format!("{formatted_amount} ({percentage})"),
            None => formatted_amount,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PriceChangeCalculator;

    #[test]
    fn test_pnl_text_keeps_the_percentage_in_brackets_and_drops_it_when_there_is_none() {
        let calculator = PriceChangeCalculator::new();
        assert_eq!(calculator.pnl_text("+$500.00".to_string(), Some("+50.00%".to_string())), "+$500.00 (+50.00%)");
        assert_eq!(calculator.pnl_text("+$500.00".to_string(), None), "+$500.00");
    }
}
