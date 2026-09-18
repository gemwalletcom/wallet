use super::model::GemFiatSuggestedAmount;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, FiatQuote, FiatQuoteType, FiatQuoteUrl};

use super::session::GemFiatSession;
use super::{GemFiatService, rules};
use crate::config::fiat_config::get_fiat_config;
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::wallet_session::GemWalletSessionService;

pub(super) const CURRENCY: Currency = Currency::USD;

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

    pub fn get_currency(&self) -> Currency {
        CURRENCY
    }

    pub fn suggested_amounts(&self, currency_symbol: String) -> Vec<GemFiatSuggestedAmount> {
        get_fiat_config()
            .suggested_amounts
            .into_iter()
            .map(|amount| GemFiatSuggestedAmount {
                amount: amount.unsigned_abs(),
                text: format!("{currency_symbol}{amount}"),
            })
            .collect()
    }

    pub fn new_session(&self, quote_type: FiatQuoteType, amount: Option<u32>) -> GemFiatSession {
        GemFiatSession::new(quote_type, amount)
    }

    pub fn random_amount(&self) -> u32 {
        rules::random_amount(&get_fiat_config())
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

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{Asset, Chain};

    use super::super::testkit::FiatQuoteTestkit;

    #[test]
    fn test_opening_a_quote_enables_the_asset_the_user_is_buying() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let testkit = FiatQuoteTestkit::new(&asset);

        let url = block_on(testkit.service.quote_url(asset.id.clone(), "quote-1".to_string())).unwrap();

        assert_eq!(url.redirect_url, "https://provider.example/checkout");
        assert_eq!(
            testkit.balances.enable_writes.lock().unwrap().clone(),
            vec![(vec![asset.id], true)],
            "a bought asset has to be visible in the wallet when the provider sends the user back"
        );
    }
}
