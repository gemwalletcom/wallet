use std::sync::Arc;

use primitives::PriceAlert;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemPriceAlertService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_price_alerts(&self, asset_id: Option<String>) -> Result<Vec<PriceAlert>, GemApiError> {
        Ok(self.api.client.get_price_alerts(asset_id).await?)
    }

    pub async fn add_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemApiError> {
        Ok(self.api.client.add_price_alerts(alerts).await?)
    }

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemApiError> {
        Ok(self.api.client.delete_price_alerts(alerts).await?)
    }
}
