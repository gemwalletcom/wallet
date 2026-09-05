use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate};

use super::model::GemPriceUpdate;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemPriceStore: Send + Sync {
    async fn get_prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemServiceError>;
    async fn get_rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemServiceError>;
    async fn save_rates(&self, rates: Vec<FiatRate>) -> Result<(), GemServiceError>;
    async fn save_prices(&self, currency: Currency, prices: Vec<GemPriceUpdate>) -> Result<(), GemServiceError>;
    async fn convert_prices(&self, currency: Currency, rate: f64) -> Result<(), GemServiceError>;
    async fn save_market(&self, asset_id: AssetId, market: AssetMarket) -> Result<(), GemServiceError>;
}
