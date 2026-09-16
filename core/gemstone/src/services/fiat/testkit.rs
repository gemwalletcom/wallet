use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetProperties, AssetScore, FiatQuote, FiatQuoteType, FiatTransactionData, Wallet, WalletId};

use super::{GemFiatQuoteRequest, GemFiatQuoteService, GemFiatQuotesResult, GemFiatService, GemFiatStore};
use crate::api::{GemApiClient, GemDeviceApiClient};
use crate::gateway::GemGateway;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::balance::GemBalanceService;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::device::GemDeviceKeyService;
use crate::services::error::GemServiceError;
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};
use crate::services::price::{GemPriceService, testkit::MemoryPriceStore};
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};
use crate::testkit::{EmptyPreferences, TestAlienProvider};

impl GemFiatQuotesResult {
    pub fn mock(quotes: Vec<FiatQuote>) -> Self {
        Self {
            request: GemFiatQuoteRequest {
                quote_type: FiatQuoteType::Buy,
                amount: 50.0,
            },
            quotes,
            error: None,
        }
    }
}

pub struct MemoryFiatStore;

#[async_trait]
impl GemFiatStore for MemoryFiatStore {
    async fn set_transactions(&self, _: WalletId, _: Vec<FiatTransactionData>) -> Result<(), GemServiceError> {
        Ok(())
    }
}

pub struct FiatQuoteTestkit {
    pub service: GemFiatQuoteService,
    pub balances: Arc<MemoryBalanceStore>,
}

impl FiatQuoteTestkit {
    pub fn new(asset: &Asset) -> Self {
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
            preferences,
            session.clone(),
        ));
        let balances = Arc::new(MemoryBalanceStore::default());
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
        Self {
            service: GemFiatQuoteService::new(fiat, balance, session),
            balances,
        }
    }
}
