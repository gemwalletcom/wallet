use primitives::{AssetFiatValue, BalanceCalculator as PrimitivesBalanceCalculator, TotalFiatValue};

#[uniffi::remote(Record)]
pub struct AssetFiatValue {
    pub amount: f64,
    pub price: f64,
    pub price_change_percentage_24h: f64,
}

#[uniffi::remote(Record)]
pub struct TotalFiatValue {
    pub value: f64,
    pub pnl_amount: f64,
    pub pnl_percentage: f64,
}

#[derive(Debug, uniffi::Object)]
pub struct BalanceCalculator {}

impl Default for BalanceCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl BalanceCalculator {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn total_fiat_value(&self, balances: Vec<AssetFiatValue>) -> TotalFiatValue {
        PrimitivesBalanceCalculator::total_fiat_value(&balances)
    }
}
