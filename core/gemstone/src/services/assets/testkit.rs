use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, Wallet, WalletId};

use super::details::GemAssetDetailsService;
use super::icon::GemAssetIconImage;
use super::{GemAssetStore, GemAssetsService};
use crate::alien::AlienProvider;
use crate::api::{GemApiClient, GemDeviceApiClient};
use crate::config::image::GemImage;
use crate::deeplink::GemDeeplinkService;
use crate::gateway::GemGateway;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::banner::GemBannerService;
use crate::services::banner::testkit::{DeniedNotificationPermissions, MemoryBannerStore};
use crate::services::device::testkit::MemoryDevicePlatform;
use crate::services::device::{GemDeviceKeyService, GemDeviceService};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::node::GemNodeService;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::services::price_alert::GemPriceAlertService;
use crate::services::price_alert::testkit::MemoryPriceAlertStore;
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::subscription::GemSubscriptionService;
use crate::services::swap::GemSwapService;
use crate::services::swap::testkit::MemorySwapStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

#[derive(Default)]
pub struct MemoryAssetStore {
    pub assets: Mutex<Vec<AssetBasic>>,
    pub added_balances: Mutex<Vec<(WalletId, Vec<AssetId>, bool)>>,
}

#[async_trait]
impl GemAssetStore for MemoryAssetStore {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(self.get_assets(asset_ids).await?.into_iter().map(|asset| asset.id).collect())
    }
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        Ok(self
            .assets
            .lock()
            .unwrap()
            .iter()
            .filter(|basic| asset_ids.contains(&basic.asset.id))
            .map(|basic| basic.asset.clone())
            .collect())
    }
    async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().extend(assets);
        Ok(())
    }
    async fn save_asset(&self, asset: AssetFull) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().push(AssetBasic::new(asset.asset, asset.properties, asset.score));
        Ok(())
    }
    async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.added_balances.lock().unwrap().push((wallet_id, asset_ids, false));
        Ok(())
    }
    async fn add_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        self.added_balances.lock().unwrap().push((wallet_id, asset_ids, enabled));
        Ok(())
    }
    async fn set_buyable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_sellable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_swappable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
    async fn set_stakeable_assets(&self, _asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        Ok(())
    }
}

impl GemAssetsService {
    pub fn mock(provider: Arc<dyn AlienProvider>, store: Arc<dyn GemAssetStore>) -> Self {
        let preferences = Arc::new(MemoryPreferencesStore::default());
        Self::new(
            Arc::new(GemApiClient::new(provider.clone())),
            Arc::new(GemGateway::new(provider, Arc::new(GemNodeService::mock()), preferences.clone(), Arc::new(EmptyPreferences))),
            store,
            Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default()))),
            Arc::new(GemPreferencesService::new(preferences)),
            Arc::new(GemWalletSessionService::new(
                Arc::new(MemoryWalletSessionStore::default()),
                Arc::new(MemoryWalletStore::default()),
            )),
        )
    }
}

impl GemAssetIconImage {
    pub fn mock_remote(asset_id: &AssetId) -> Self {
        Self::Remote {
            url: GemImage::Asset { asset_id: asset_id.clone() }.url(),
        }
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
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        let device_api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let device = Arc::new(GemDeviceService::new(
            device_api.clone(),
            Arc::new(GemSubscriptionService::new(device_api.clone(), discovery.wallets.clone())),
            discovery.wallets.clone(),
            Arc::new(MemoryDevicePlatform),
            preferences.clone(),
        ));
        let swap = Arc::new(GemSwapService::mock(Arc::new(MemorySwapStore::default())));
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
                Arc::new(MemoryPriceAlertStore::default()),
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
