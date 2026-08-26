use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, AssetPrice, Markets};

use crate::api::{GemApiClient, GemApiError};

#[derive(Debug, uniffi::Object)]
pub struct GemPriceService {
    api: Arc<GemApiClient>,
}

#[uniffi::export]
impl GemPriceService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_prices(&self, currency: Option<Currency>, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemApiError> {
        Ok(self.api.client.get_prices(currency, asset_ids).await?)
    }

    pub async fn get_markets(&self) -> Result<Markets, GemApiError> {
        Ok(self.api.client.get_markets().await?)
    }
}
