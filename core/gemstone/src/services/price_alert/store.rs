use async_trait::async_trait;
use primitives::{AssetId, PriceAlert};

use super::error::GemPriceAlertError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemPriceAlertStore: Send + Sync {
    async fn get_price_alerts(&self, asset_id: Option<AssetId>) -> Result<Vec<PriceAlert>, GemPriceAlertError>;
    async fn update(&self, alerts: Vec<PriceAlert>, delete_ids: Vec<String>) -> Result<(), GemPriceAlertError>;
}
