use std::sync::Arc;

use primitives::{FiatQuote, FiatQuoteType, FiatQuoteUrl, FiatTransactionData};

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

    pub async fn get_quotes(&self, wallet_id: String, quote_type: FiatQuoteType, asset_id: String, amount: f64, currency: String) -> Result<Vec<FiatQuote>, GemApiError> {
        Ok(self.api.client.get_fiat_quotes(wallet_id, quote_type, asset_id, amount, currency).await?.quotes)
    }

    pub async fn get_quote_url(&self, wallet_id: String, quote_id: String) -> Result<FiatQuoteUrl, GemApiError> {
        Ok(self.api.client.get_fiat_quote_url(wallet_id, quote_id).await?)
    }

    pub async fn get_transactions(&self, wallet_id: String) -> Result<Vec<FiatTransactionData>, GemApiError> {
        Ok(self.api.client.get_fiat_transactions(wallet_id).await?)
    }
}
