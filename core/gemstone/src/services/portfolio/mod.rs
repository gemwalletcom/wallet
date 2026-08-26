use std::sync::Arc;

use primitives::{ChartPeriod, PortfolioAssets, PortfolioAssetsRequest};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemPortfolioService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemPortfolioService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_assets(&self, period: ChartPeriod, request: PortfolioAssetsRequest) -> Result<PortfolioAssets, GemApiError> {
        Ok(self.api.client.get_portfolio_assets(period, request).await?)
    }
}
