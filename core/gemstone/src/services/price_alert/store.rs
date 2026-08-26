use async_trait::async_trait;
use primitives::PriceAlert;

use super::error::GemPriceAlertError;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemPriceAlertStore: Send + Sync {
    async fn get_price_alerts(&self, asset_id: Option<String>) -> Result<Vec<PriceAlert>, GemPriceAlertError>;
    async fn apply(&self, delete_ids: Vec<String>, alerts: Vec<PriceAlert>) -> Result<(), GemPriceAlertError>;
}
