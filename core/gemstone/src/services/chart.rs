use std::sync::Arc;

use primitives::{AssetId, ChartPeriod, Charts};

use crate::api::{GemApiClient, GemApiError};

#[derive(Debug, uniffi::Object)]
pub struct GemChartService {
    api: Arc<GemApiClient>,
}

#[uniffi::export]
impl GemChartService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_charts(&self, asset_id: AssetId, period: ChartPeriod) -> Result<Charts, GemApiError> {
        Ok(self.api.client.get_charts(asset_id, period).await?)
    }
}
