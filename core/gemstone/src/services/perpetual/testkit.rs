use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::perpetual::PerpetualData;
use primitives::{PerpetualMarketData, PerpetualPosition, PerpetualProvider, Wallet, WalletId};

use super::{GemPerpetualService, GemPerpetualStore};
use crate::api::GemApiClient;
use crate::gateway::{EmptyPreferences, GemGateway};
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::balance::GemBalanceService;
use crate::services::balance::testkit::RecordingBalanceStore;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::transfer::GemRecentActivityService;
use crate::services::transfer::testkit::MemoryRecentActivityStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::TestAlienProvider;

#[derive(Default)]
pub struct MemoryPerpetualStore {
    pub positions: Mutex<Vec<PerpetualPosition>>,
    pub position_writes: Mutex<Vec<(Vec<PerpetualPosition>, Vec<String>)>>,
    pub markets: Mutex<Vec<PerpetualMarketData>>,
    pub price_writes: Mutex<Vec<HashMap<String, f64>>>,
    pub deleted: Mutex<u32>,
}

#[async_trait]
impl GemPerpetualStore for MemoryPerpetualStore {
    async fn save_perpetuals(&self, _: Vec<PerpetualData>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_pinned(&self, _: Vec<String>, _: bool) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn delete_perpetuals(&self) -> Result<(), GemServiceError> {
        *self.deleted.lock().unwrap() += 1;
        Ok(())
    }
    async fn get_positions(&self, _: WalletId, _: PerpetualProvider) -> Result<Vec<PerpetualPosition>, GemServiceError> {
        Ok(self.positions.lock().unwrap().clone())
    }
    async fn get_position_ids(&self, _: WalletId, _: PerpetualProvider) -> Result<Vec<String>, GemServiceError> {
        Ok(self.positions.lock().unwrap().iter().map(|position| position.id.clone()).collect())
    }
    async fn update_positions(&self, _: WalletId, positions: Vec<PerpetualPosition>, delete_ids: Vec<String>) -> Result<(), GemServiceError> {
        self.position_writes.lock().unwrap().push((positions, delete_ids));
        Ok(())
    }
    async fn update_market(&self, market: PerpetualMarketData) -> Result<(), GemServiceError> {
        self.markets.lock().unwrap().push(market);
        Ok(())
    }
    async fn update_prices(&self, prices: HashMap<String, f64>) -> Result<(), GemServiceError> {
        self.price_writes.lock().unwrap().push(prices);
        Ok(())
    }
}

pub struct PerpetualTestkit {
    pub service: GemPerpetualService,
    pub provider: Arc<TestAlienProvider>,
    pub store: Arc<MemoryPerpetualStore>,
    pub wallets: Arc<MemoryWalletStore>,
    pub balances: Arc<RecordingBalanceStore>,
    pub preferences: Arc<GemPreferencesService>,
    pub wallet_preferences: Arc<GemWalletPreferencesService>,
    pub wallet_id: WalletId,
}

impl PerpetualTestkit {
    pub fn new() -> Self {
        let wallet = Wallet::mock();
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let wallets = Arc::new(MemoryWalletStore {
            wallets: Mutex::new(vec![wallet.clone()]),
            ..Default::default()
        });
        let session = Arc::new(GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore {
                current: Mutex::new(Some(wallet.id.clone())),
            }),
            wallets.clone(),
        ));
        let provider = Arc::new(TestAlienProvider::with_status(503));
        let gateway = Arc::new(GemGateway::new(provider.clone(), preferences_store, Arc::new(EmptyPreferences)));
        let price = Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default())));
        let asset_store = Arc::new(MemoryAssetStore::default());
        let assets = Arc::new(GemAssetsService::new(
            Arc::new(GemApiClient::new(provider.clone())),
            gateway.clone(),
            asset_store.clone(),
            price.clone(),
            preferences.clone(),
            session.clone(),
        ));
        let balances = Arc::new(RecordingBalanceStore::default());
        let balance = Arc::new(GemBalanceService::new(
            gateway.clone(),
            wallets.clone(),
            asset_store.clone(),
            balances.clone(),
            assets,
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
        ));
        let wallet_preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let store = Arc::new(MemoryPerpetualStore::default());
        let service = GemPerpetualService::new(
            gateway,
            price,
            store.clone(),
            asset_store,
            preferences.clone(),
            balance,
            wallet_preferences.clone(),
            session.clone(),
            Arc::new(GemRecentActivityService::new(Arc::new(MemoryRecentActivityStore::default()), session)),
        );
        Self {
            service,
            provider,
            store,
            wallets,
            balances,
            preferences,
            wallet_preferences,
            wallet_id: wallet.id,
        }
    }
}

impl Default for PerpetualTestkit {
    fn default() -> Self {
        Self::new()
    }
}
