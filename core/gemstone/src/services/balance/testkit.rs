use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use num_bigint::BigUint;
use primitives::{AssetId, Chain, WalletId};

use super::model::{GemAssetConfiguration, GemBalanceRecord, GemBalanceUpdate, GemBalanceUpdateType};
use super::store::GemBalanceStore;
use super::{GemAssetBalance, GemBalanceService};
use crate::api::GemApiClient;
use crate::gateway::GemGateway;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::error::GemServiceError;
use crate::services::node::GemNodeService;
use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};
use crate::services::price::{GemPriceService, testkit::MemoryPriceStore};
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};
use crate::testkit::{EmptyPreferences, TestAlienProvider};

impl GemAssetBalance {
    pub fn mock() -> Self {
        Self::zero(AssetId::from_chain(Chain::Ethereum))
    }

    pub fn mock_with_available(available: u64) -> Self {
        Self {
            available: BigUint::from(available),
            ..Self::mock()
        }
    }
}

impl GemBalanceUpdate {
    pub fn mock(update_type: GemBalanceUpdateType) -> Self {
        Self {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            update_type,
            is_active: true,
        }
    }
}

#[derive(Default)]
pub struct YieldOnce {
    yielded: bool,
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            return Poll::Ready(());
        }
        self.yielded = true;
        context.waker().wake_by_ref();
        Poll::Pending
    }
}

#[derive(Default)]
pub struct MemoryBalanceStore {
    pub balances: Mutex<HashMap<WalletId, Vec<GemAssetBalance>>>,
    pub enabled_asset_ids: Mutex<HashMap<WalletId, Vec<AssetId>>>,
    pub requests: Mutex<Vec<WalletId>>,
    pub balance_writes: Mutex<Vec<Vec<GemBalanceRecord>>>,
    pub enable_writes: Mutex<Vec<(Vec<AssetId>, bool)>>,
    pub configuration_writes: Mutex<Vec<(Vec<AssetId>, GemAssetConfiguration)>>,
    pub yields_between_read_and_write: bool,
}

impl MemoryBalanceStore {
    pub fn with_balances(wallet_id: WalletId, balances: Vec<GemAssetBalance>) -> Self {
        Self {
            balances: Mutex::new(HashMap::from([(wallet_id, balances)])),
            ..Default::default()
        }
    }

    pub fn with_enabled_asset_ids(wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Self {
        Self {
            enabled_asset_ids: Mutex::new(HashMap::from([(wallet_id, asset_ids)])),
            ..Default::default()
        }
    }
}

#[async_trait::async_trait]
impl GemBalanceStore for MemoryBalanceStore {
    async fn get_available_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
        self.requests.lock().unwrap().push(wallet_id.clone());
        let stored = self.balances.lock().unwrap().get(&wallet_id).into_iter().flatten().filter(|balance| asset_ids.contains(&balance.asset_id)).cloned().collect();
        if self.yields_between_read_and_write {
            YieldOnce::default().await;
        }
        Ok(stored)
    }
    async fn update_balances(&self, wallet_id: WalletId, balances: Vec<GemBalanceRecord>) -> Result<(), GemServiceError> {
        let mut stored = self.balances.lock().unwrap();
        let wallet = stored.entry(wallet_id).or_default();
        for record in &balances {
            let balance = GemAssetBalance::from(record.clone());
            match wallet.iter().position(|stored| stored.asset_id == balance.asset_id) {
                Some(index) => wallet[index] = balance,
                None => wallet.push(balance),
            }
        }
        drop(stored);
        self.balance_writes.lock().unwrap().push(balances);
        Ok(())
    }
    async fn get_enabled_asset_ids(&self, wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.enabled_asset_ids.lock().unwrap().get(&wallet_id).cloned().unwrap_or_default())
    }
    async fn set_asset_configuration(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, configuration: GemAssetConfiguration) -> Result<(), GemServiceError> {
        self.configuration_writes.lock().unwrap().push((asset_ids.clone(), configuration));
        let Some(enabled) = configuration.is_enabled else {
            return Ok(());
        };
        self.enable_writes.lock().unwrap().push((asset_ids.clone(), enabled));
        let mut wallets = self.enabled_asset_ids.lock().unwrap();
        let stored = wallets.entry(wallet_id).or_default();
        if enabled {
            stored.extend(asset_ids);
        } else {
            stored.retain(|asset_id| !asset_ids.contains(asset_id));
        }
        Ok(())
    }
}

pub struct BalanceTestkit {
    pub service: GemBalanceService,
    pub assets: Arc<MemoryAssetStore>,
    pub balances: Arc<MemoryBalanceStore>,
}

impl BalanceTestkit {
    pub fn new(balances: MemoryBalanceStore) -> Self {
        let provider = Arc::new(TestAlienProvider::with_status(503));
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let gateway = Arc::new(GemGateway::new(provider.clone(), Arc::new(GemNodeService::mock()), preferences_store, Arc::new(EmptyPreferences)));
        let wallets = Arc::new(MemoryWalletStore::default());
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), wallets.clone()));
        let assets = Arc::new(MemoryAssetStore::default());
        let assets_service = Arc::new(GemAssetsService::new(
            Arc::new(GemApiClient::new(provider)),
            gateway.clone(),
            assets.clone(),
            Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default()))),
            preferences,
            session.clone(),
        ));
        let balances = Arc::new(balances);
        let service = GemBalanceService::new(gateway, balances.clone(), assets_service, session, Arc::new(SubscriptionTestkit::new(&[], &[]).service));
        Self { service, assets, balances }
    }

    pub fn next_sequence(&self) -> u64 {
        self.service.next_sequence()
    }
}
