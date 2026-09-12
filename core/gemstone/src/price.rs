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

    pub fn percentage(&self, from: f64, to: f64) -> f64 {
        Calculator::percentage(from, to)
    }

    pub fn pnl_percentage(&self, pnl: f64, margin: f64) -> f64 {
        Calculator::pnl_percentage(pnl, margin)
    }

    pub fn amount(&self, percentage: f64, value: f64) -> f64 {
        Calculator::amount(percentage, value)
    }
}
