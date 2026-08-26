use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, FiatQuote, FiatQuoteType, FiatQuoteUrl, FiatTransactionData, WalletId};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemFiatService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemFiatService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_quotes(&self, wallet_id: WalletId, quote_type: FiatQuoteType, asset_id: AssetId, amount: f64, currency: Currency) -> Result<Vec<FiatQuote>, GemApiError> {
        Ok(self
            .api
            .client
            .get_fiat_quotes(wallet_id.id(), quote_type, asset_id.to_string(), amount, currency.to_string())
            .await?
            .quotes)
    }

    pub async fn get_quote_url(&self, wallet_id: WalletId, quote_id: String) -> Result<FiatQuoteUrl, GemApiError> {
        Ok(self.api.client.get_fiat_quote_url(wallet_id.id(), quote_id).await?)
    }

    pub async fn get_transactions(&self, wallet_id: WalletId) -> Result<Vec<FiatTransactionData>, GemApiError> {
        Ok(self.api.client.get_fiat_transactions(wallet_id.id()).await?)
    }
}
