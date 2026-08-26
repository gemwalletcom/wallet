pub mod error;
pub mod store;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, FiatQuote, FiatQuoteType, FiatQuoteUrl, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;

pub use error::GemFiatError;
pub use store::GemFiatStore;

#[derive(uniffi::Object)]
pub struct GemFiatService {
    api: Arc<GemDeviceApiClient>,
    assets: Arc<GemAssetsService>,
    store: Arc<dyn GemFiatStore>,
}

#[uniffi::export]
impl GemFiatService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, assets: Arc<GemAssetsService>, store: Arc<dyn GemFiatStore>) -> Self {
        Self { api, assets, store }
    }

    pub async fn sync_transactions(&self, wallet_id: WalletId) -> Result<(), GemFiatError> {
        let transactions = self.api.client.get_fiat_transactions(wallet_id.id()).await.map_err(GemApiError::from)?;
        let asset_ids = transactions.iter().map(|data| data.transaction.asset_id.clone()).collect();
        self.assets.prefetch_assets(asset_ids).await?;
        self.store.save_transactions(wallet_id, transactions).await
    }

    pub async fn get_quotes(&self, wallet_id: WalletId, quote_type: FiatQuoteType, asset_id: AssetId, amount: f64, currency: Currency) -> Result<Vec<FiatQuote>, GemFiatError> {
        Ok(self
            .api
            .client
            .get_fiat_quotes(wallet_id.id(), quote_type, asset_id.to_string(), amount, currency.to_string())
            .await
            .map_err(GemApiError::from)?
            .quotes)
    }

    pub async fn get_quote_url(&self, wallet_id: WalletId, quote_id: String) -> Result<FiatQuoteUrl, GemFiatError> {
        Ok(self.api.client.get_fiat_quote_url(wallet_id.id(), quote_id).await.map_err(GemApiError::from)?)
    }
}
