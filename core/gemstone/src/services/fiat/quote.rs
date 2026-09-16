use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, FiatQuote, FiatQuoteType, FiatQuoteUrl};

use super::model::GemFiatAmountCheck;
use super::session::GemFiatSession;
use super::{GemFiatService, rules};
use crate::config::fiat_config::get_fiat_config;
use crate::models::custom_types::GemBigUint;
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

    pub fn suggested_amounts(&self) -> Vec<i32> {
        get_fiat_config().suggested_amounts
    }

    pub fn new_session(&self, quote_type: FiatQuoteType, amount: Option<u32>) -> GemFiatSession {
        GemFiatSession::new(quote_type, amount)
    }

    pub fn random_amount(&self) -> u32 {
        rules::random_amount(&get_fiat_config())
    }

    pub fn amount_check(&self, quote_type: FiatQuoteType, amount: f64, quote: Option<FiatQuote>, available: GemBigUint) -> GemFiatAmountCheck {
        rules::amount_check(&get_fiat_config(), quote_type, amount, quote.as_ref(), &available, CURRENCY)
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
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use futures::executor::block_on;
    use primitives::{Asset, AssetBasic, AssetProperties, AssetScore, Chain, FiatTransactionData, Wallet, WalletId};

    use super::super::{GemFiatService, GemFiatStore};
    use super::*;
    use crate::api::{GemApiClient, GemDeviceApiClient};
    use crate::gateway::{EmptyPreferences, GemGateway};
    use crate::services::assets::GemAssetsService;
    use crate::services::assets::testkit::MemoryAssetStore;
    use crate::services::balance::GemBalanceService;
    use crate::services::balance::testkit::RecordingBalanceStore;
    use crate::services::device::GemDeviceKeyService;
    use crate::services::preferences::GemPreferencesService;
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::services::price::GemPriceService;
    use crate::services::price::testkit::MemoryPriceStore;
    use crate::services::stream::testkit::SubscriptionTestkit;
    use crate::services::wallet::testkit::MemoryWalletStore;
    use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
    use crate::services::wallet_session::GemWalletSessionService;
    use crate::testkit::TestAlienProvider;

    #[derive(Default)]
    struct MemoryFiatStore;

    #[async_trait]
    impl GemFiatStore for MemoryFiatStore {
        async fn set_transactions(&self, _: WalletId, _: Vec<FiatTransactionData>) -> Result<(), GemServiceError> {
            Ok(())
        }
    }

    fn service(asset: &Asset) -> (GemFiatQuoteService, Arc<RecordingBalanceStore>) {
        let wallet = Wallet::mock();
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let wallets = Arc::new(MemoryWalletStore {
            wallets: Mutex::new(vec![wallet.clone()]),
            ..Default::default()
        });
        let session = Arc::new(GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore {
                current: Mutex::new(Some(wallet.id)),
            }),
            wallets.clone(),
        ));
        let provider = Arc::new(TestAlienProvider::with_json(200, r#"{"redirectUrl":"https://provider.example/checkout"}"#));
        let gateway = Arc::new(GemGateway::new(provider.clone(), preferences_store, Arc::new(EmptyPreferences)));
        let asset_store = Arc::new(MemoryAssetStore {
            assets: Mutex::new(vec![AssetBasic::new(asset.clone(), AssetProperties::default(asset.id.clone()), AssetScore::default())]),
            ..Default::default()
        });
        let assets = Arc::new(GemAssetsService::new(
            Arc::new(GemApiClient::new(provider.clone())),
            gateway.clone(),
            asset_store.clone(),
            Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default()))),
            preferences.clone(),
            session.clone(),
        ));
        let balances = Arc::new(RecordingBalanceStore::default());
        let balance = Arc::new(GemBalanceService::new(
            gateway,
            wallets,
            asset_store,
            balances.clone(),
            assets.clone(),
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
        ));
        let fiat = Arc::new(GemFiatService::new(
            Arc::new(GemDeviceApiClient::new(provider, Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))),
            assets,
            Arc::new(MemoryFiatStore),
        ));
        (GemFiatQuoteService::new(fiat, balance, session), balances)
    }

    #[test]
    fn test_opening_a_quote_enables_the_asset_the_user_is_buying() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let (service, balances) = service(&asset);

        let url = block_on(service.quote_url(asset.id.clone(), "quote-1".to_string())).unwrap();

        assert_eq!(url.redirect_url, "https://provider.example/checkout");
        assert_eq!(
            balances.enable_writes.lock().unwrap().clone(),
            vec![(vec![asset.id], true)],
            "a bought asset has to be visible in the wallet when the provider sends the user back"
        );
    }
}
