use std::sync::Arc;

use async_trait::async_trait;
use primitives::{AssetId, PriceAlert, Wallet};

use super::details::GemAssetDetailsService;
use crate::api::GemDeviceApiClient;
use crate::deeplink::GemDeeplinkService;
use crate::gateway::EmptyPreferences;
use crate::gem_swapper::GemSwapper;
use crate::keystore::GemKeystore;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::banner::testkit::MemoryBannerStore;
use crate::services::banner::{GemBannerService, GemNotificationPermissions};
use crate::services::device::testkit::MemoryDevicePlatform;
use crate::services::device::{GemDeviceKeyService, GemDeviceService};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price_alert::{GemPriceAlertService, GemPriceAlertStore};
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::subscription::GemSubscriptionService;
use crate::services::swap::GemSwapService;
use crate::services::swap::testkit::MemorySwapStore;
use crate::services::wallet::testkit::MemoryKeystorePassword;
use crate::testkit::TestAlienProvider;

pub struct EmptyPriceAlertStore;

#[async_trait]
impl GemPriceAlertStore for EmptyPriceAlertStore {
    async fn get_price_alerts(&self, _: Option<AssetId>) -> Result<Vec<PriceAlert>, GemServiceError> {
        Ok(Vec::new())
    }
    async fn update_price_alerts(&self, _: Vec<PriceAlert>, _: Vec<String>) -> Result<(), GemServiceError> {
        Ok(())
    }
}

pub struct DeniedNotificationPermissions;

#[async_trait]
impl GemNotificationPermissions for DeniedNotificationPermissions {
    fn is_available(&self) -> bool {
        false
    }
    async fn request_permissions_or_open_settings(&self) -> Result<bool, GemServiceError> {
        Ok(false)
    }
}

pub struct AssetDetailsTestkit {
    pub service: GemAssetDetailsService,
    pub provider: Arc<TestAlienProvider>,
    pub discovery: DiscoveryTestkit,
}

impl AssetDetailsTestkit {
    pub fn with_status(status: u16) -> Self {
        Self::with_bodies(status, &[])
    }

    pub fn with_bodies(status: u16, bodies: &[(&str, &str)]) -> Self {
        let provider = Arc::new(TestAlienProvider::with_json_by_path(status, bodies));
        let discovery = DiscoveryTestkit::with_provider(provider.clone(), Wallet::mock());
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let device = Arc::new(GemDeviceService::new(
            device_api.clone(),
            Arc::new(GemSubscriptionService::new(device_api.clone(), discovery.wallets.clone())),
            discovery.wallets.clone(),
            Arc::new(MemoryDevicePlatform),
            preferences.clone(),
        ));
        let swap = Arc::new(GemSwapService::new(
            Arc::new(GemSwapper::new(provider.clone(), preferences_store)),
            GemKeystore::new(std::env::temp_dir().to_string_lossy().to_string()).unwrap(),
            Arc::new(MemoryKeystorePassword::default()),
            Arc::new(MemorySwapStore::default()),
        ));
        let service = GemAssetDetailsService::new(
            discovery.assets.clone(),
            discovery.balance.clone(),
            discovery.transactions.clone(),
            Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()))),
            swap,
            Arc::new(GemExplorerService::new(preferences.clone())),
            Arc::new(GemPriceAlertService::new(
                device_api,
                preferences,
                Arc::new(EmptyPriceAlertStore),
                device,
                Arc::new(DeniedNotificationPermissions),
            )),
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
            Arc::new(GemDeeplinkService::new()),
            discovery.session.clone(),
        );
        Self { service, provider, discovery }
    }
}
