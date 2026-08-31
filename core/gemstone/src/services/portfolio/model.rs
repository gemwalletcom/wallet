use primitives::currency::Currency;
use primitives::{Chain, ChartDateValue, ChartPeriod, ChartValuePercentage, WalletId};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioValues {
    pub values: Vec<ChartDateValue>,
    pub all_time_high: Option<ChartValuePercentage>,
    pub all_time_low: Option<ChartValuePercentage>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPortfolioDataInput {
    Wallet { wallet_id: WalletId, period: ChartPeriod, currency: Currency },
    Perpetuals { chain: Chain, address: String, period: ChartPeriod },
}
