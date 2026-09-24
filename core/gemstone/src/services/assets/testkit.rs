use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, AssetProperties, AssetRank, AssetScore, Wallet, WalletId};

use super::details::GemAssetDetailsService;
use super::icon::GemAssetIconImage;
use super::{GemAssetFilter, GemAssetStore, GemAssetsService};
use crate::alien::AlienProvider;
use crate::api::{GemApiClient, GemDeviceApiClient};
use crate::config::image::GemImage;
use crate::deeplink::GemDeeplinkService;
use crate::gateway::GemGateway;
use crate::services::asset_discovery::testkit::DiscoveryTestkit;
use crate::services::banner::GemBannerService;
use crate::services::banner::testkit::{DeniedNotificationPermissions, MemoryBannerStore};
use crate::services::device::GemDeviceKeyService;
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
use crate::services::swap::GemSwapService;
use crate::services::swap::testkit::MemorySwapStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

fn as_stored(basic: AssetBasic) -> AssetBasic {
    AssetBasic::new(
        basic.asset,
        AssetProperties { has_price: false, ..basic.properties },
        AssetScore {
            rank: basic.score.rank,
            rank_type: AssetRank::Unknown,
        },
    )
}

#[derive(Default)]
pub struct MemoryAssetStore {
    pub assets: Mutex<Vec<AssetBasic>>,
    pub id_reads: Mutex<usize>,
    pub filtered_asset_ids: Mutex<Vec<AssetId>>,
    pub wallet_asset_filters: Mutex<Vec<Vec<GemAssetFilter>>>,
    pub asset_writes: Mutex<Vec<Vec<AssetBasic>>>,
    pub added_balances: Mutex<Vec<(WalletId, Vec<AssetId>, bool)>>,
    pub buyable_writes: Mutex<Vec<Vec<AssetId>>>,
    pub sellable_writes: Mutex<Vec<Vec<AssetId>>>,
    pub swappable_writes: Mutex<Vec<Vec<AssetId>>>,
    pub stakeable_writes: Mutex<Vec<Vec<AssetId>>>,
}

#[async_trait]
impl GemAssetStore for MemoryAssetStore {
    async fn get_asset_ids(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        *self.id_reads.lock().unwrap() += 1;
        Ok(self.get_assets(asset_ids).await?.into_iter().map(|asset| asset.id).collect())
    }
    async fn get_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        Ok(self.assets.lock().unwrap().iter().filter(|basic| asset_ids.contains(&basic.asset.id)).map(|basic| basic.asset.clone()).collect())
    }
    async fn get_asset_basics(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetBasic>, GemServiceError> {
        Ok(self.assets.lock().unwrap().iter().filter(|basic| asset_ids.contains(&basic.asset.id)).cloned().collect())
    }
    async fn get_wallet_assets(&self, _wallet_id: WalletId, filters: Vec<GemAssetFilter>) -> Result<Vec<Asset>, GemServiceError> {
        let filtered = self.filtered_asset_ids.lock().unwrap().clone();
        let assets = self
            .assets
            .lock()
            .unwrap()
            .iter()
            .filter(|basic| filters.is_empty() || filtered.contains(&basic.asset.id))
            .map(|basic| basic.asset.clone())
            .collect();
        self.wallet_asset_filters.lock().unwrap().push(filters);
        Ok(assets)
    }
    async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemServiceError> {
        self.asset_writes.lock().unwrap().push(assets.clone());
        let mut stored = self.assets.lock().unwrap();
        for basic in assets.into_iter().map(as_stored) {
            match stored.iter_mut().find(|current| current.asset.id == basic.asset.id) {
                Some(current) => *current = basic,
                None => stored.push(basic),
            }
        }
        Ok(())
    }
    async fn save_asset(&self, asset: AssetFull) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().push(as_stored(AssetBasic::new(asset.asset, asset.properties, asset.score)));
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
    async fn set_buyable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.buyable_writes.lock().unwrap().push(asset_ids);
        Ok(())
    }
    async fn set_sellable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.sellable_writes.lock().unwrap().push(asset_ids);
        Ok(())
    }
    async fn set_swappable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.swappable_writes.lock().unwrap().push(asset_ids);
        Ok(())
    }
    async fn set_stakeable_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.stakeable_writes.lock().unwrap().push(asset_ids);
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
            Arc::new(GemPriceService::mock(Arc::new(MemoryPriceStore::default()))),
            Arc::new(GemPreferencesService::new(preferences)),
            Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), Arc::new(MemoryWalletStore::default()))),
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
        let swap = Arc::new(GemSwapService::mock(Arc::new(MemorySwapStore::default())));
        let service = GemAssetDetailsService::new(
            discovery.assets.clone(),
            discovery.balance.clone(),
            discovery.transactions.clone(),
            Arc::new(GemBannerService::new(Arc::new(MemoryBannerStore::default()), primitives::Platform::IOS)),
            swap,
            Arc::new(GemExplorerService::new(preferences.clone())),
            Arc::new(GemPriceAlertService::new(device_api, preferences, Arc::new(MemoryPriceAlertStore::default()), Arc::new(DeniedNotificationPermissions))),
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
            Arc::new(GemDeeplinkService::new()),
            discovery.session.clone(),
        );
        Self { service, provider, discovery }
    }
}
