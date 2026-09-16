use std::sync::{Arc, Mutex};

use primitives::{Transaction, Wallet, WalletId};

use super::GemAssetDiscoveryService;
use crate::api::{GemApiClient, GemDeviceApiClient};
use crate::gateway::GemGateway;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::balance::GemBalanceService;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::device::GemDeviceKeyService;
use crate::services::nft::GemNftService;
use crate::services::nft::testkit::MemoryNftStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::transaction_state::GemTransactionStatusService;
use crate::services::transactions::GemTransactionsService;
use crate::services::transactions::testkit::MemoryTransactionStore;
use crate::services::wallet::testkit::{MemoryAddressStore, MemoryWalletStore};
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

#[derive(Default)]
pub struct RecordingTransactionStatus {
    pub tracked: Mutex<Vec<Vec<Transaction>>>,
}

impl GemTransactionStatusService for RecordingTransactionStatus {
    fn track(&self, _: WalletId, transactions: Vec<Transaction>) {
        self.tracked.lock().unwrap().push(transactions);
    }
}

pub struct DiscoveryTestkit {
    pub discovery: Arc<GemAssetDiscoveryService>,
    pub assets: Arc<GemAssetsService>,
    pub transactions: Arc<GemTransactionsService>,
    pub wallets: Arc<MemoryWalletStore>,
    pub balance: Arc<GemBalanceService>,
    pub provider: Arc<TestAlienProvider>,
    pub balances: Arc<MemoryBalanceStore>,
    pub preferences: Arc<GemPreferencesService>,
    pub wallet_preferences: Arc<GemWalletPreferencesService>,
    pub session: Arc<GemWalletSessionService>,
    pub wallet_id: WalletId,
}

impl DiscoveryTestkit {
    pub fn with_response(provider: TestAlienProvider) -> Self {
        Self::with_provider(Arc::new(provider), Wallet::mock())
    }

    pub fn with_provider(provider: Arc<TestAlienProvider>, wallet: Wallet) -> Self {
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
        let gateway = Arc::new(GemGateway::new(provider.clone(), preferences_store, Arc::new(EmptyPreferences)));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let asset_store = Arc::new(MemoryAssetStore::default());
        let assets: Arc<GemAssetsService> = Arc::new(GemAssetsService::new(
            Arc::new(GemApiClient::new(provider.clone())),
            gateway.clone(),
            asset_store.clone(),
            Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default()))),
            preferences.clone(),
            session.clone(),
        ));
        let balances = Arc::new(MemoryBalanceStore::default());
        let balance = Arc::new(GemBalanceService::new(
            gateway,
            wallets.clone(),
            asset_store,
            balances.clone(),
            assets.clone(),
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
        ));
        let wallet_preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let transactions = Arc::new(GemTransactionsService::new(
            device_api.clone(),
            assets.clone(),
            Arc::new(MemoryTransactionStore::default()),
            Arc::new(MemoryAddressStore::default()),
            wallet_preferences.clone(),
            preferences.clone(),
            session.clone(),
            Arc::new(RecordingTransactionStatus::default()),
        ));
        let nft = Arc::new(GemNftService::new(device_api.clone(), Arc::new(MemoryNftStore::default()), session.clone()));
        let discovery = Arc::new(GemAssetDiscoveryService::new(
            device_api.clone(),
            balance.clone(),
            transactions.clone(),
            nft,
            wallets.clone(),
            wallet_preferences.clone(),
        ));
        Self {
            discovery,
            assets,
            transactions,
            wallets,
            balance,
            provider,
            balances,
            preferences,
            wallet_preferences,
            session,
            wallet_id: wallet.id,
        }
    }

    pub fn with_status(status: u16) -> Self {
        Self::with_response(TestAlienProvider::with_status(status))
    }
}
