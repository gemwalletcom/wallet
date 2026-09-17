use async_trait::async_trait;
use primitives::{AssetId, Chain, Currency, PriceAlert};

use super::session::GemPriceAlertSession;
use super::store::GemPriceAlertStore;
use crate::services::error::GemServiceError;

impl GemPriceAlertSession {
    pub fn mock() -> Self {
        Self::new(AssetId::from_chain(Chain::Ethereum), Currency::USD).on_price(Some(100.0))
    }
}

#[derive(Default)]
pub struct MemoryPriceAlertStore {
    pub alerts: Vec<PriceAlert>,
}

#[async_trait]
impl GemPriceAlertStore for MemoryPriceAlertStore {
    async fn get_price_alerts(&self, _: Option<AssetId>) -> Result<Vec<PriceAlert>, GemServiceError> {
        Ok(self.alerts.clone())
    }
    async fn update_price_alerts(&self, _: Vec<PriceAlert>, _: Vec<String>) -> Result<(), GemServiceError> {
        Ok(())
    }
}
