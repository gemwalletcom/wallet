use std::sync::Arc;

use primitives::{AssetId, Feature, FiatQuote, FiatQuoteType, FiatQuoteUrl};

use super::model::GemFiatSuggestedAmount;
use super::session::{GemFiatQuoteRequest, GemFiatQuotesResult, GemFiatSession};
use super::{GemFiatService, rules};
use crate::config::fiat_config::get_fiat_config;
use crate::constants::FIAT_QUOTE_CURRENCY;
use crate::formatted_number::GemFormattedNumber;
use crate::models::state::GemLoadState;
use crate::services::balance::GemBalanceService;
use crate::services::config::GemConfigService;
use crate::services::error::GemServiceError;
use crate::services::transfer::GemRecentActivityService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemFiatQuoteService {
    fiat: Arc<GemFiatService>,
    balances: Arc<GemBalanceService>,
    session: Arc<GemWalletSessionService>,
    recent_activity: Arc<GemRecentActivityService>,
    config: Arc<GemConfigService>,
}

#[uniffi::export]
impl GemFiatQuoteService {
    #[uniffi::constructor]
    pub fn new(fiat: Arc<GemFiatService>, balances: Arc<GemBalanceService>, session: Arc<GemWalletSessionService>, recent_activity: Arc<GemRecentActivityService>, config: Arc<GemConfigService>) -> Self {
        Self {
            fiat,
            balances,
            session,
            recent_activity,
            config,
        }
    }

    pub fn is_available(&self, quote_type: FiatQuoteType) -> bool {
        let feature = match quote_type {
            FiatQuoteType::Buy => Feature::Buy,
            FiatQuoteType::Sell => Feature::Sell,
        };
        self.config.is_feature_enabled(feature)
    }

    pub fn suggested_amounts(&self) -> Vec<GemFiatSuggestedAmount> {
        get_fiat_config()
            .suggested_amounts
            .into_iter()
            .map(|amount| GemFiatSuggestedAmount {
                amount: amount.unsigned_abs(),
                value: GemFormattedNumber::whole_currency(f64::from(amount), FIAT_QUOTE_CURRENCY),
            })
            .collect()
    }

    pub fn new_session(&self, quote_type: FiatQuoteType, amount: Option<u32>) -> GemFiatSession {
        GemFiatSession::new(quote_type, amount)
    }

    pub fn random_amount(&self) -> u32 {
        rules::random_amount(&get_fiat_config())
    }

    pub async fn refresh_transactions(&self) -> GemLoadState {
        GemLoadState::of(&self.sync_transactions().await)
    }

    pub async fn quotes(&self, request: GemFiatQuoteRequest, asset_id: AssetId) -> GemFiatQuotesResult {
        let (quotes, error) = match self.fetch_quotes(request.quote_type, asset_id, request.amount).await {
            Ok(quotes) => (quotes, None),
            Err(error) => (vec![], Some(error)),
        };
        GemFiatQuotesResult { request, quotes, error }
    }

    pub async fn quote_url(&self, asset_id: AssetId, quote_id: String) -> Result<FiatQuoteUrl, GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        let url = self.fiat.get_quote_url(wallet_id.clone(), quote_id).await?;
        self.balances.enable_assets(wallet_id, vec![asset_id]).await?;
        Ok(url)
    }
}

impl GemFiatQuoteService {
    async fn fetch_quotes(&self, quote_type: FiatQuoteType, asset_id: AssetId, amount: f64) -> Result<Vec<FiatQuote>, GemServiceError> {
        let wallet_id = self.session.current_wallet_id()?;
        if let Ok(asset) = self.fiat.asset(asset_id.clone()).await {
            let _ = self.recent_activity.add_recent(rules::quote_action(&quote_type), asset).await;
        }
        self.fiat.get_quotes(wallet_id, quote_type, asset_id, amount, FIAT_QUOTE_CURRENCY).await
    }

    async fn sync_transactions(&self) -> Result<(), GemServiceError> {
        self.fiat.sync_transactions(self.session.current_wallet_id()?).await
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{Asset, Chain};

    use super::super::GemFiatQuoteRequest;
    use super::super::testkit::FiatQuoteTestkit;

    fn request(quote_type: primitives::FiatQuoteType) -> GemFiatQuoteRequest {
        GemFiatQuoteRequest { quote_type, amount: 50.0 }
    }

    #[test]
    fn test_asking_for_quotes_records_what_the_user_is_buying_or_selling() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let testkit = FiatQuoteTestkit::new(&asset);

        block_on(testkit.service.quotes(request(primitives::FiatQuoteType::Sell), asset.id.clone()));
        block_on(testkit.service.quotes(request(primitives::FiatQuoteType::Buy), asset.id));

        let recorded: Vec<primitives::RecentActivityType> = testkit.recents.added.lock().unwrap().iter().map(|(activity, _)| activity.activity_type.clone()).collect();
        assert_eq!(
            recorded,
            vec![primitives::RecentActivityType::FiatSell, primitives::RecentActivityType::FiatBuy],
            "each quote request records the side the user asked for"
        );
    }

    #[test]
    fn test_a_failed_quote_request_answers_the_request_it_was_asked() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let testkit = FiatQuoteTestkit::new(&asset);

        let result = block_on(testkit.service.quotes(request(primitives::FiatQuoteType::Sell), asset.id));

        assert_eq!(result.request, request(primitives::FiatQuoteType::Sell));
        assert!(result.quotes.is_empty());
        assert!(result.error.is_some(), "the provider answered with something that is not a quote list");
    }

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
