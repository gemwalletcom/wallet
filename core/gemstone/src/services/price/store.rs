use async_trait::async_trait;
use primitives::FiatRate;
use primitives::currency::Currency;

use super::error::GemPriceError;
use super::model::GemPriceUpdate;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemPriceStore: Send + Sync {
    async fn get_rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemPriceError>;
    async fn save_rates(&self, rates: Vec<FiatRate>) -> Result<(), GemPriceError>;
    async fn save_prices(&self, currency: Currency, prices: Vec<GemPriceUpdate>) -> Result<(), GemPriceError>;
    async fn convert_prices(&self, currency: Currency, rate: f64) -> Result<(), GemPriceError>;
}
