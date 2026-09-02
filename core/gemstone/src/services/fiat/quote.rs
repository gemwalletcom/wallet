use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, FiatQuote, FiatQuoteType, FiatQuoteUrl};

use super::model::GemFiatAmountCheck;
use super::{GemFiatService, rules};
use crate::config::fiat_config::{FiatConfig, get_fiat_config};
use crate::models::custom_types::GemBigUint;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::wallet_session::GemWalletSessionService;

const CURRENCY: Currency = Currency::USD;

#[derive(uniffi::Object)]
pub struct GemFiatQuoteService {
    fiat: Arc<GemFiatService>,
    balances: Arc<GemBalanceService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemFiatQuoteService {
    #[uniffi::constructor]
    pub fn new(fiat: Arc<GemFiatService>, balances: Arc<GemBalanceService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { fiat, balances, session }
    }

    pub fn currency(&self) -> Currency {
        CURRENCY
    }

    pub fn config(&self) -> FiatConfig {
        get_fiat_config()
    }

    pub fn default_amount(&self, quote_type: FiatQuoteType) -> u32 {
        rules::default_amount(&get_fiat_config(), quote_type)
    }

    pub fn random_amount(&self) -> u32 {
        rules::random_amount(&get_fiat_config())
    }

    pub fn amount_check(&self, quote_type: FiatQuoteType, amount: f64, quote: Option<FiatQuote>, available: GemBigUint) -> GemFiatAmountCheck {
        rules::amount_check(&get_fiat_config(), quote_type, amount, quote.as_ref(), &available)
    }

    pub fn quote_debounce_milliseconds(&self) -> u64 {
        self.fiat.quote_debounce_milliseconds()
    }

    pub fn quote_refresh_interval_milliseconds(&self) -> u64 {
        self.fiat.quote_refresh_interval_milliseconds()
    }

    pub async fn sync_transactions(&self) -> Result<(), GemServiceError> {
        self.fiat.sync_transactions(self.session.current_wallet_id()?).await
    }

    pub async fn quotes(&self, quote_type: FiatQuoteType, asset_id: AssetId, amount: f64) -> Result<Vec<FiatQuote>, GemServiceError> {
        self.fiat.get_quotes(self.session.current_wallet_id()?, quote_type, asset_id, amount, CURRENCY).await
    }

    pub async fn quote_url(&self, asset_id: AssetId, quote_id: String) -> Result<FiatQuoteUrl, GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        let url = self.fiat.get_quote_url(wallet_id.clone(), quote_id).await?;
        self.balances.set_assets_enabled(wallet_id, vec![asset_id], true).await?;
        Ok(url)
    }
}
