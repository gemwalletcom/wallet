pub mod model;
pub mod quote;
pub mod rules;
pub mod session;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, FiatQuote, FiatQuoteType, FiatQuoteUrl, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;

pub use model::GemFiatAmountCheck;
pub use quote::GemFiatQuoteService;
pub use session::{GemFiatButtonAction, GemFiatButtonState, GemFiatOperation, GemFiatQuotePhase, GemFiatQuoteRequest, GemFiatQuotesResult, GemFiatSession};
pub use store::GemFiatStore;

const QUOTE_DEBOUNCE_MILLISECONDS: u64 = 250;
const QUOTE_REFRESH_INTERVAL_MILLISECONDS: u64 = 5 * 60 * 1_000;

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
}

impl GemFiatService {
    pub fn quote_debounce_milliseconds(&self) -> u64 {
        QUOTE_DEBOUNCE_MILLISECONDS
    }

    pub fn quote_refresh_interval_milliseconds(&self) -> u64 {
        QUOTE_REFRESH_INTERVAL_MILLISECONDS
    }

    pub async fn sync_transactions(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let transactions = self.api.client.get_fiat_transactions(wallet_id.id()).await.map_err(GemApiError::from)?;
        let asset_ids = transactions.iter().map(|data| data.transaction.asset_id.clone()).collect();
        self.assets.sync_missing_assets(asset_ids).await?;
        self.store.save_transactions(wallet_id, transactions).await
    }

    pub async fn get_quotes(&self, wallet_id: WalletId, quote_type: FiatQuoteType, asset_id: AssetId, amount: f64, currency: Currency) -> Result<Vec<FiatQuote>, GemServiceError> {
        Ok(self
            .api
            .client
            .get_fiat_quotes(wallet_id.id(), quote_type, asset_id.to_string(), amount, currency.to_string())
            .await
            .map_err(GemApiError::from)?
            .quotes)
    }

    pub async fn get_quote_url(&self, wallet_id: WalletId, quote_id: String) -> Result<FiatQuoteUrl, GemServiceError> {
        Ok(self.api.client.get_fiat_quote_url(wallet_id.id(), quote_id).await.map_err(GemApiError::from)?)
    }
}
