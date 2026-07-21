#[derive(Debug, uniffi::Object)]
pub struct PriceChangeCalculator {}

#[uniffi::export]
impl PriceChangeCalculator {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn percentage(&self, from: f64, to: f64) -> f64 {
        primitives::PriceChangeCalculator::percentage(from, to)
    }

    pub fn amount(&self, percentage: f64, value: f64) -> f64 {
        primitives::PriceChangeCalculator::amount(percentage, value)
    }
}
