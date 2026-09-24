use std::sync::Arc;

use primitives::Wallet;

use super::GemAppStartService;
use crate::api::{GemApiClient, GemDeviceApiClient};
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::banner::GemBannerService;
use crate::services::banner::testkit::MemoryBannerStore;
use crate::services::config::GemConfigService;
use crate::services::device::testkit::MemoryDevicePlatform;
use crate::services::device::{GemDeviceKeyService, GemDeviceService};
use crate::services::file::testkit::NoopFileStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::subscription::GemSubscriptionService;
use crate::services::support::GemSupportService;
use crate::services::support::testkit::MemorySupportStore;
use crate::services::wallet::testkit::{OTHER_PHRASE, PHRASE, WalletTestkit};
use crate::services::wallet_configuration::GemWalletConfigurationService;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

pub struct AppStartTestkit {
    pub service: GemAppStartService,
    pub support: Arc<MemorySupportStore>,
    pub wallets: WalletTestkit,
    pub banners: Arc<MemoryBannerStore>,
    pub assets: Arc<MemoryAssetStore>,
    pub first: Wallet,
    pub second: Wallet,
}

impl AppStartTestkit {
    pub async fn new() -> Self {
        let provider = Arc::new(TestAlienProvider::with_json_by_path(200, &[]));
        let wallets = WalletTestkit::new();
        let first = wallets.import("First", PHRASE).await;
        let second = wallets.import("Second", OTHER_PHRASE).await;
        let discovery = DiscoveryTestkit::with_provider(provider.clone(), first.clone());
        *discovery.wallets.wallets.lock().unwrap() = vec![first.clone(), second.clone()];
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        let device_key = Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), device_key));
        let banner_store = Arc::new(MemoryBannerStore::default());
        let banners = Arc::new(GemBannerService::new(banner_store.clone(), primitives::Platform::IOS));
        let support_store = Arc::new(MemorySupportStore::default());
        let service = GemAppStartService::new(
            Arc::new(GemConfigService::new(Arc::new(GemApiClient::new(provider.clone())), preferences.clone())),
            banners.clone(),
            discovery.assets.clone(),
            discovery.balance.clone(),
            Arc::new(GemWalletConfigurationService::new(device_api.clone(), banners.clone(), discovery.wallet_preferences.clone())),
            wallets.service.clone(),
            Arc::new(GemDeviceService::new(
                device_api.clone(),
                Arc::new(GemSubscriptionService::new(device_api.clone(), discovery.session.clone())),
                discovery.session.clone(),
                Arc::new(MemoryDevicePlatform),
                preferences,
            )),
            Arc::new(GemSupportService::new(device_api, support_store.clone(), Arc::new(NoopFileStore), provider.clone())),
        );
        Self {
            service,
            support: support_store,
            wallets,
            banners: banner_store,
            assets: discovery.asset_store.clone(),
            first,
            second,
        }
    }
}
