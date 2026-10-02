use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use primitives::{AssetId, AssetList, PerpetualId, Wallet};

use super::{GemSearchService, GemSearchStore};
use crate::api::GemApiClient;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::error::GemServiceError;
use crate::services::perpetual::testkit::PerpetualTestkit;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::testkit::TestAlienProvider;

#[derive(Default)]
pub struct MemorySearchStore {
    pub assets: Mutex<Vec<(String, Vec<AssetId>)>>,
    pub perpetuals: Mutex<Vec<(String, Vec<PerpetualId>)>>,
    pub lists: Mutex<Vec<(String, Vec<AssetList>)>>,
}

#[async_trait]
impl GemSearchStore for MemorySearchStore {
    async fn set_assets(&self, key: String, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        self.assets.lock().unwrap().push((key, asset_ids));
        Ok(())
    }
    async fn set_perpetuals(&self, key: String, perpetual_ids: Vec<PerpetualId>) -> Result<(), GemServiceError> {
        self.perpetuals.lock().unwrap().push((key, perpetual_ids));
        Ok(())
    }
    async fn set_lists(&self, key: String, lists: Vec<AssetList>) -> Result<(), GemServiceError> {
        self.lists.lock().unwrap().push((key, lists));
        Ok(())
    }
}

pub struct SearchTestkit {
    pub service: GemSearchService,
    pub provider: Arc<TestAlienProvider>,
    pub wallet: Wallet,
    pub asset_store: Arc<MemoryAssetStore>,
    pub prices: Arc<MemoryPriceStore>,
    pub balances: Arc<MemoryBalanceStore>,
    pub store: Arc<MemorySearchStore>,
}

impl SearchTestkit {
    pub fn with_status(status: u16) -> Self {
        Self::with_provider(TestAlienProvider::with_status(status))
    }

    pub fn with_bodies(bodies: &[(&str, &str)]) -> Self {
        Self::with_provider(TestAlienProvider::with_json_by_path(200, bodies))
    }

    fn with_provider(provider: TestAlienProvider) -> Self {
        let perpetuals = PerpetualTestkit::with_provider(provider);
        let prices = Arc::new(MemoryPriceStore::default());
        let store = Arc::new(MemorySearchStore::default());
        let service = GemSearchService::new(
            Arc::new(GemApiClient::new(perpetuals.provider.clone())),
            Arc::new(GemAssetsService::mock_with_price_store(perpetuals.provider.clone(), perpetuals.asset_store.clone(), prices.clone())),
            perpetuals.balance.clone(),
            Arc::new(GemPriceService::mock(prices.clone())),
            Arc::new(perpetuals.service),
            store.clone(),
        );
        Self {
            service,
            provider: perpetuals.provider,
            wallet: Wallet::mock(),
            asset_store: perpetuals.asset_store,
            prices,
            balances: perpetuals.balances,
            store,
        }
    }
}
