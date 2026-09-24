use std::sync::{Arc, Mutex};

use primitives::{Wallet, WalletId};

use super::GemAssetDiscoveryService;
use crate::api::GemStaticApiClient;
use crate::api::{GemApiClient, GemDeviceApiClient};
use crate::gateway::GemGateway;
use crate::payment::GemPaymentService;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::balance::GemBalanceService;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::device::GemDeviceKeyService;
use crate::services::explorer::GemExplorerService;
use crate::services::name::GemNameService;
use crate::services::nft::GemNftService;
use crate::services::nft::testkit::MemoryNftStore;
use crate::services::node::GemNodeService;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::services::stake::GemStakeService;
use crate::services::stake::testkit::UnusedStakeStore;
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::transaction_state::GemTransactionStateService;
use crate::services::transaction_state::testkit::{MemoryTransactionStateStore, RecordingTransactionStatus};
use crate::services::transactions::GemTransactionsService;
use crate::services::wallet::testkit::{MemoryAddressStore, MemoryWalletStore};
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

pub struct DiscoveryTestkit {
    pub discovery: Arc<GemAssetDiscoveryService>,
    pub state: Arc<GemTransactionStateService>,
    pub status: Arc<RecordingTransactionStatus>,
    pub assets: Arc<GemAssetsService>,
    pub asset_store: Arc<MemoryAssetStore>,
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
        let gateway = Arc::new(GemGateway::new(provider.clone(), Arc::new(GemNodeService::mock()), preferences_store, Arc::new(EmptyPreferences)));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let asset_store = Arc::new(MemoryAssetStore::default());
        let assets: Arc<GemAssetsService> = Arc::new(GemAssetsService::new(
            Arc::new(GemApiClient::new(provider.clone())),
            gateway.clone(),
            asset_store.clone(),
            Arc::new(GemPriceService::mock(Arc::new(MemoryPriceStore::default()))),
            preferences.clone(),
            session.clone(),
        ));
        let balances = Arc::new(MemoryBalanceStore::default());
        let balance = Arc::new(GemBalanceService::new(gateway.clone(), balances.clone(), assets.clone(), session.clone(), Arc::new(SubscriptionTestkit::new(&[], &[]).service)));
        let names = Arc::new(GemNameService::new(device_api.clone(), Arc::new(MemoryAddressStore::default())));
        let wallet_preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let transactions = Arc::new(GemTransactionsService::new(
            device_api.clone(),
            assets.clone(),
            Arc::new(MemoryTransactionStateStore::default()),
            names.clone(),
            wallet_preferences.clone(),
            session.clone(),
            Arc::new(RecordingTransactionStatus::default()),
        ));
        let nft = Arc::new(GemNftService::new(device_api.clone(), Arc::new(MemoryNftStore::default()), session.clone()));
        let status = Arc::new(RecordingTransactionStatus::default());
        let state = Arc::new(GemTransactionStateService::new(
            gateway.clone(),
            Arc::new(MemoryTransactionStateStore::default()),
            assets.clone(),
            balance.clone(),
            Arc::new(GemStakeService::new(
                gateway.clone(),
                Arc::new(GemStaticApiClient::new(provider.clone())),
                Arc::new(UnusedStakeStore),
                names.clone(),
                Arc::new(GemExplorerService::new(preferences.clone())),
                preferences.clone(),
                session.clone(),
            )),
            nft.clone(),
            Arc::new(GemPaymentService::new(provider.clone(), assets.clone())),
        ));
        state.set_status(status.clone());
        let discovery = Arc::new(GemAssetDiscoveryService::new(device_api.clone(), balance.clone(), transactions.clone(), nft, session.clone(), wallet_preferences.clone()));
        Self {
            discovery,
            state,
            status,
            assets,
            asset_store,
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
