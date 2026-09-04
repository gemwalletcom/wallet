pub mod model;
pub mod rules;
pub mod store;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Chain, ChartPeriod, PortfolioAssets, PortfolioAssetsRequest, PortfolioData, PortfolioType, Wallet, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
use crate::services::stream::rules::hyperliquid_account;

pub use model::GemPortfolioValues;
pub use store::GemPortfolioStore;

#[derive(uniffi::Object)]
pub struct GemPortfolioService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemPortfolioStore>,
    price: Arc<GemPriceService>,
    perpetual: Arc<GemPerpetualService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemPortfolioService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        store: Arc<dyn GemPortfolioStore>,
        price: Arc<GemPriceService>,
        perpetual: Arc<GemPerpetualService>,
        preferences: Arc<GemPreferencesService>,
    ) -> Self {
        Self {
            api,
            store,
            price,
            perpetual,
            preferences,
        }
    }

    pub async fn portfolio_data(&self, wallet: Wallet, portfolio_type: PortfolioType, period: ChartPeriod) -> Result<PortfolioData, GemServiceError> {
        match portfolio_type {
            PortfolioType::Wallet => Ok(rules::wallet_portfolio_data(
                self.sync_wallet_values(wallet.id, period, self.preferences.get_currency()).await?,
            )),
            PortfolioType::Perpetuals => {
                let account = hyperliquid_account(&wallet.accounts).ok_or(GemServiceError::NotFound {
                    msg: "wallet has no perpetual account".to_string(),
                })?;
                Ok(rules::perpetual_portfolio_data(
                    self.perpetual.get_portfolio(Chain::HyperCore, account.address.clone()).await?,
                    period,
                ))
            }
        }
    }
}

impl GemPortfolioService {
    async fn sync_wallet_values(&self, wallet_id: WalletId, period: ChartPeriod, currency: Currency) -> Result<GemPortfolioValues, GemServiceError> {
        let portfolio = self.get_wallet_assets(wallet_id, period).await?;
        let rate = self.price.rate(currency.clone()).await?.ok_or(GemServiceError::InvalidInput {
            msg: format!("unknown currency: {currency}"),
        })?;
        Ok(rules::converted_portfolio(portfolio, rate.rate))
    }

    async fn get_assets(&self, period: ChartPeriod, request: PortfolioAssetsRequest) -> Result<PortfolioAssets, GemApiError> {
        Ok(self.api.client.get_portfolio_assets(period, request).await?)
    }

    async fn get_wallet_assets(&self, wallet_id: WalletId, period: ChartPeriod) -> Result<PortfolioAssets, GemServiceError> {
        let assets = self.store.get_wallet_assets(wallet_id).await?;
        Ok(self.get_assets(period, PortfolioAssetsRequest { assets }).await?)
    }
}
