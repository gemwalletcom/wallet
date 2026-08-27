pub mod model;
pub mod rules;
pub mod store;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{ChartPeriod, PortfolioAssets, PortfolioAssetsRequest, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::error::GemServiceError;
use crate::services::price::GemPriceService;

pub use model::GemPortfolioValues;
pub use store::GemPortfolioStore;

#[derive(uniffi::Object)]
pub struct GemPortfolioService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemPortfolioStore>,
    price: Arc<GemPriceService>,
}

#[uniffi::export]
impl GemPortfolioService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemPortfolioStore>, price: Arc<GemPriceService>) -> Self {
        Self { api, store, price }
    }

    pub async fn sync_wallet_values(&self, wallet_id: WalletId, period: ChartPeriod, currency: Currency) -> Result<GemPortfolioValues, GemServiceError> {
        let portfolio = self.get_wallet_assets(wallet_id, period).await?;
        let rate = self
            .price
            .rate(currency.clone())
            .await?
            .ok_or(GemServiceError::UnknownCurrency { currency: currency.to_string() })?;
        Ok(rules::converted_portfolio(portfolio, rate.rate))
    }

    pub async fn get_assets(&self, period: ChartPeriod, request: PortfolioAssetsRequest) -> Result<PortfolioAssets, GemApiError> {
        Ok(self.api.client.get_portfolio_assets(period, request).await?)
    }

    pub async fn get_wallet_assets(&self, wallet_id: WalletId, period: ChartPeriod) -> Result<PortfolioAssets, GemServiceError> {
        let assets = self.store.get_wallet_assets(wallet_id).await?;
        Ok(self.get_assets(period, PortfolioAssetsRequest { assets }).await?)
    }
}
