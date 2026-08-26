pub mod store;

use std::sync::Arc;

use primitives::{ChartPeriod, PortfolioAssets, PortfolioAssetsRequest, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::error::GemServiceError;

pub use store::GemPortfolioStore;

#[derive(uniffi::Object)]
pub struct GemPortfolioService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemPortfolioStore>,
}

#[uniffi::export]
impl GemPortfolioService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemPortfolioStore>) -> Self {
        Self { api, store }
    }

    pub async fn get_assets(&self, period: ChartPeriod, request: PortfolioAssetsRequest) -> Result<PortfolioAssets, GemApiError> {
        Ok(self.api.client.get_portfolio_assets(period, request).await?)
    }

    pub async fn get_wallet_assets(&self, wallet_id: WalletId, period: ChartPeriod) -> Result<PortfolioAssets, GemServiceError> {
        let assets = self.store.get_wallet_assets(wallet_id).await?;
        Ok(self.get_assets(period, PortfolioAssetsRequest { assets }).await?)
    }
}
