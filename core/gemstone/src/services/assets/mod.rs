pub mod add;
pub mod config;
pub mod details;
pub mod icon;
pub mod model;
pub mod rules;
pub mod selection;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{Asset, AssetBasic, AssetFull, AssetId, AssetPrice, Chain, ConfigVersions, FiatAssets, FiatQuoteType, SearchResponse, Wallet, WalletId};

pub use add::GemAddAssetService;
pub use details::GemAssetDetailsService;
pub use model::{
    AssetList, GemAssetAction, GemAssetDetails, GemAssetDetailsInput, GemAssetDetailsState, GemAssetFilter, GemAssetNetworkDestination, GemHeaderButton, GemHeaderButtonKind, GemPriceRow, GemSelectAssetFlow, GemSelectAssetType,
    GemSelectRowAction, GemWalletSearchLimits,
};
pub use selection::GemAssetSelectionService;
pub use store::GemAssetStore;

use crate::api::{GemApiClient, GemApiError};
use crate::gateway::GemGateway;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemAssetsService {
    api: Arc<GemApiClient>,
    gateway: Arc<GemGateway>,
    store: Arc<dyn GemAssetStore>,
    price: Arc<GemPriceService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemAssetsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, gateway: Arc<GemGateway>, store: Arc<dyn GemAssetStore>, price: Arc<GemPriceService>, preferences: Arc<GemPreferencesService>, session: Arc<GemWalletSessionService>) -> Self {
        Self {
            api,
            gateway,
            store,
            price,
            preferences,
            session,
        }
    }

    pub async fn ensure_asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        if let Some(asset) = self.stored_asset(&asset_id).await? {
            return Ok(asset);
        }
        self.sync_missing_assets(vec![asset_id.clone()]).await?;
        self.stored_asset(&asset_id).await?.ok_or_else(|| GemServiceError::NotFound { msg: format!("asset not found: {asset_id}") })
    }

    pub async fn open_asset(&self, asset_id: AssetId) -> Result<Option<Asset>, GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        self.open_wallet_asset(wallet, asset_id).await
    }
}

impl GemAssetsService {
    pub async fn wallet_assets(&self, wallet_id: WalletId, filters: Vec<GemAssetFilter>) -> Result<Vec<Asset>, GemServiceError> {
        self.store.get_wallet_assets(wallet_id, filters).await
    }

    pub async fn ensure_token_asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        if asset_id.is_native_mirror() {
            return Err(GemServiceError::Unsupported {
                msg: format!("{asset_id} mirrors the native coin"),
            });
        }
        match self.ensure_asset(asset_id.clone()).await {
            Ok(asset) => Ok(asset),
            Err(error) if asset_id.is_native() => Err(error),
            Err(_) => self.node_token_asset(asset_id).await,
        }
    }

    pub async fn open_wallet_asset(&self, wallet: Wallet, asset_id: AssetId) -> Result<Option<Asset>, GemServiceError> {
        if !rules::can_open(&wallet, &asset_id) {
            return Ok(None);
        }
        let asset = match self.ensure_asset(asset_id.clone()).await {
            Ok(asset) => asset,
            Err(GemServiceError::NotFound { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        self.add_missing_balances(wallet.id, vec![asset_id]).await?;
        Ok(Some(asset))
    }

    async fn sync_asset(&self, asset_id: AssetId) -> Result<AssetFull, GemServiceError> {
        let asset = self.get_asset(asset_id.clone()).await?;
        self.store.save_asset(asset.clone()).await?;
        let price = asset.price.as_ref().map(|price| AssetPrice::new(asset_id.clone(), price.price, price.price_change_percentage_24h, price.updated_at));
        self.price.update_asset_price(asset_id.clone(), price).await?;
        if let Some(market) = asset.market.clone() {
            self.price.update_market(asset_id, market).await?;
        }
        Ok(asset)
    }

    pub async fn save_assets(&self, assets: Vec<AssetBasic>) -> Result<(), GemServiceError> {
        if assets.is_empty() {
            return Ok(());
        }
        let stored = self.store.get_asset_basics(assets.iter().map(|basic| basic.asset.id.clone()).collect()).await?;
        let changed = rules::changed_assets(assets, &stored);
        if changed.is_empty() {
            return Ok(());
        }
        self.store.save_assets(changed).await
    }

    pub async fn sync_asset_associations(&self, asset_id: AssetId) -> Result<Vec<AssetId>, GemServiceError> {
        let asset = self.sync_asset(asset_id).await?;
        let associations: Vec<AssetId> = asset.associations.into_iter().map(|association| association.asset_id).collect();
        if !associations.is_empty() {
            self.sync_missing_assets(associations.clone()).await?;
        }
        Ok(associations)
    }

    pub(crate) async fn prepare_for_action(&self, action: GemAssetAction, asset_id: AssetId) -> Result<(), GemServiceError> {
        match action {
            GemAssetAction::Receive => self.sync_asset_associations(asset_id).await.map(|_| ()),
            GemAssetAction::Open | GemAssetAction::Send | GemAssetAction::Buy | GemAssetAction::Sell | GemAssetAction::SwapPay | GemAssetAction::SwapReceive => Ok(()),
        }
    }

    pub async fn sync_missing_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        let existing = self.store.get_asset_ids(asset_ids.clone()).await?;
        let missing = rules::missing_asset_ids(asset_ids, existing);
        if missing.is_empty() {
            return Ok(vec![]);
        }
        self.sync_assets(missing).await
    }

    async fn sync_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        let assets = self.api.client.get_assets(asset_ids, None).await.map_err(GemApiError::from)?;
        let asset_ids = assets.iter().map(|asset| asset.asset.id.clone()).collect();
        self.store.save_assets(assets).await?;
        Ok(asset_ids)
    }

    pub(crate) async fn ensure_simulation_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        let existing = self.store.get_asset_ids(asset_ids.clone()).await?;
        let missing = rules::missing_asset_ids(asset_ids.clone(), existing);
        if missing.is_empty() {
            return self.assets(asset_ids).await;
        }
        let synced = self.sync_assets(missing.clone()).await.unwrap_or_default();
        for asset_id in rules::missing_asset_ids(missing, synced) {
            if self.node_token_asset(asset_id).await.is_err() {
                continue;
            }
        }
        self.assets(asset_ids).await
    }

    async fn node_token_asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        let Some(token_id) = asset_id.token_id.clone() else {
            return Err(GemServiceError::NotFound { msg: format!("asset not found: {asset_id}") });
        };
        let asset = self.gateway.get_token_data(asset_id.chain, token_id).await?;
        self.store.save_assets(vec![rules::default_asset_basic(asset.clone())]).await?;
        Ok(asset)
    }

    pub async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        if asset_ids.is_empty() {
            return Ok(());
        }
        let stored = self.store.get_asset_ids(asset_ids).await?;
        if stored.is_empty() {
            return Ok(());
        }
        self.store.add_missing_balances(wallet_id, stored).await
    }

    pub async fn add_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        if asset_ids.is_empty() {
            return Ok(());
        }
        self.store.add_balances(wallet_id, asset_ids, enabled).await
    }

    pub async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        self.store.get_assets(asset_ids).await
    }

    pub async fn get_asset(&self, asset_id: AssetId) -> Result<AssetFull, GemApiError> {
        Ok(self.api.client.get_asset(asset_id).await?)
    }

    pub async fn search_assets(&self, query: String, chains: Vec<Chain>) -> Result<Vec<AssetBasic>, GemApiError> {
        Ok(self.api.client.get_search_assets(query, chains).await?)
    }

    pub async fn search(&self, query: String, chains: Vec<Chain>, tags: Vec<String>) -> Result<SearchResponse, GemApiError> {
        Ok(self.api.client.get_search(query, chains, tags).await?)
    }

    pub async fn sync_availability(&self, versions: ConfigVersions) -> Result<(), GemServiceError> {
        let results = futures::future::join_all(rules::asset_list_versions(&versions).into_iter().map(|(list, remote_version)| self.sync_availability_list(list, remote_version))).await;
        for result in results {
            result?;
        }
        Ok(())
    }

    async fn sync_availability_list(&self, list: AssetList, remote_version: i32) -> Result<(), GemServiceError> {
        if !rules::is_asset_list_outdated(self.preferences.get_assets_version(list).as_deref(), remote_version) {
            return Ok(());
        }
        let assets = match list {
            AssetList::Buy => self.get_fiat_assets(FiatQuoteType::Buy).await?,
            AssetList::Sell => self.get_fiat_assets(FiatQuoteType::Sell).await?,
            AssetList::Swap => self.get_swap_assets().await?,
        };
        let asset_ids = rules::asset_ids(&assets.asset_ids);
        self.sync_missing_assets(asset_ids.clone()).await?;
        match list {
            AssetList::Buy => self.store.set_buyable_assets(asset_ids).await?,
            AssetList::Sell => self.store.set_sellable_assets(asset_ids).await?,
            AssetList::Swap => self.store.set_swappable_assets(rules::swappable_asset_ids(asset_ids)).await?,
        }
        self.preferences.set_assets_version(list, assets.version.to_string())
    }

    pub async fn search_tokens(&self, token_id: String, chains: Vec<Chain>) -> Vec<AssetBasic> {
        let lookups = chains.into_iter().filter(|chain| chain.default_asset_type().is_some()).map(|chain| {
            let token_id = token_id.clone();
            async move {
                if self.gateway.get_is_token_address(chain, token_id.clone()).await.ok()? {
                    let asset = self.gateway.get_token_data(chain, token_id).await.ok()?;
                    (!asset.id.is_native_mirror()).then(|| rules::default_asset_basic(asset))
                } else {
                    None
                }
            }
        });
        futures::future::join_all(lookups).await.into_iter().flatten().collect()
    }

    pub async fn sync_default_assets(&self) -> Result<(), GemServiceError> {
        self.ensure_default_assets().await?;
        self.store.set_stakeable_assets(rules::stakeable_asset_ids()).await
    }

    pub async fn ensure_default_assets(&self) -> Result<(), GemServiceError> {
        let assets = rules::default_assets();
        let existing = self.store.get_asset_ids(assets.iter().map(|asset| asset.asset.id.clone()).collect()).await?;
        let missing = rules::missing_assets(assets, existing);
        if missing.is_empty() {
            return Ok(());
        }
        self.store.save_assets(missing).await
    }

    pub async fn search_assets_and_tokens(&self, query: String, chains: Vec<Chain>) -> Result<Vec<AssetBasic>, GemServiceError> {
        let token_chains = rules::token_search_chains(&chains);
        let (assets, tokens) = futures::join!(self.search_assets(query.clone(), chains), self.search_tokens(query, token_chains));
        Ok(rules::merge_assets(assets?, tokens))
    }

    pub async fn get_fiat_assets(&self, quote_type: FiatQuoteType) -> Result<FiatAssets, GemApiError> {
        Ok(self.api.client.get_fiat_assets(quote_type).await?)
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, GemApiError> {
        Ok(self.api.client.get_swap_assets().await?)
    }

    async fn stored_asset(&self, asset_id: &AssetId) -> Result<Option<Asset>, GemServiceError> {
        Ok(self.assets(vec![asset_id.clone()]).await?.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::assets::testkit::MemoryAssetStore;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use primitives::AssetRank;
    use primitives::asset_constants::{ARC_USDC_TOKEN_ID, ETHEREUM_USDT_ASSET_ID};

    #[test]
    fn test_saving_assets_writes_only_what_the_store_does_not_already_hold() {
        block_on(async {
            let store = Arc::new(MemoryAssetStore::default());
            let service = GemAssetsService::mock(Arc::new(TestAlienProvider::with_status(503)), store.clone());
            let mut ethereum = rules::default_asset_basic(Asset::from_chain(Chain::Ethereum));
            ethereum.properties.has_price = true;
            assert_ne!(ethereum.score.rank_type, AssetRank::Unknown, "the backend names a rank type the apps never receive");

            service.save_assets(vec![ethereum.clone()]).await.unwrap();
            service.save_assets(vec![ethereum.clone()]).await.unwrap();

            assert_eq!(store.asset_writes.lock().unwrap().len(), 1, "an identical sync writes no rows");

            let mut reranked = ethereum.clone();
            reranked.score.rank += 5;
            service.save_assets(vec![reranked.clone(), ethereum.clone()]).await.unwrap();

            let writes = store.asset_writes.lock().unwrap();
            assert_eq!(writes.len(), 2);
            assert_eq!(writes[1], vec![reranked], "only the row that changed is written");
        })
    }

    #[test]
    fn test_a_simulation_review_opens_when_a_token_cannot_be_read() {
        block_on(async {
            let ethereum = Asset::from_chain(Chain::Ethereum);
            let store = Arc::new(MemoryAssetStore {
                assets: std::sync::Mutex::new(vec![rules::default_asset_basic(ethereum.clone())]),
                ..MemoryAssetStore::default()
            });
            let service = GemAssetsService::mock(Arc::new(TestAlienProvider::offline()), store.clone());
            let unverified = AssetId::from_token(Chain::Ethereum, "0x1234567890123456789012345678901234567890");

            let assets = service.ensure_simulation_assets(vec![ethereum.id.clone(), unverified]).await.unwrap();

            assert_eq!(assets, vec![ethereum]);
            assert!(store.asset_writes.lock().unwrap().is_empty(), "an unreadable token is not stored");
        })
    }

    #[test]
    fn test_only_a_receive_pick_prefetches_the_asset_associations() {
        block_on(async {
            let token = AssetId::from_token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7");

            let sent = Arc::new(TestAlienProvider::offline());
            let service = GemAssetsService::mock(sent.clone(), Arc::new(MemoryAssetStore::default()));
            service.prepare_for_action(GemAssetAction::Send, token.clone()).await.unwrap();
            assert!(sent.requested_paths().is_empty(), "sending needs no associations");

            let received = Arc::new(TestAlienProvider::offline());
            let service = GemAssetsService::mock(received.clone(), Arc::new(MemoryAssetStore::default()));
            let _ = service.prepare_for_action(GemAssetAction::Receive, token).await;
            assert!(!received.requested_paths().is_empty(), "receiving asks for the asset and its associations");
        })
    }

    #[test]
    fn test_a_simulation_token_is_checked_once_and_asked_for_once() {
        block_on(async {
            let provider = Arc::new(TestAlienProvider::with_json(200, USDT_RESPONSE));
            let store = Arc::new(MemoryAssetStore::default());
            let service = GemAssetsService::mock(provider.clone(), store.clone());
            let usdt = AssetId::from_token(Chain::Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7");

            let assets = service.ensure_simulation_assets(vec![usdt.clone()]).await.unwrap();

            assert_eq!(assets.iter().map(|asset| asset.id.clone()).collect::<Vec<_>>(), vec![usdt]);
            assert_eq!(*store.id_reads.lock().unwrap(), 1, "the existence read is not repeated");
            assert_eq!(provider.requested_paths().len(), 1, "one backend request covers the missing token");
        })
    }

    const USDT_RESPONSE: &str = r#"[{
        "asset": {"id": "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", "name": "Tether", "symbol": "USDT", "decimals": 6, "type": "ERC20"},
        "properties": {"isEnabled": true, "isBuyable": true, "isSellable": true, "isSwapable": true, "isStakeable": false, "isEarnable": true, "earnApr": 4.68, "hasImage": true, "hasPrice": true},
        "score": {"rank": 34, "type": "low"},
        "price": null
    }]"#;

    #[test]
    fn test_ensure_token_asset_keeps_the_backend_properties() {
        block_on(async {
            let provider = Arc::new(TestAlienProvider::with_json(200, USDT_RESPONSE));
            let store = Arc::new(MemoryAssetStore::default());
            let service = GemAssetsService::mock(provider.clone(), store.clone());

            let asset = service.ensure_token_asset(ETHEREUM_USDT_ASSET_ID.clone()).await.unwrap();

            let saved = store.assets.lock().unwrap().clone();
            assert_eq!(asset.symbol, "USDT");
            assert_eq!(saved.len(), 1);
            assert_eq!((saved[0].score.rank, saved[0].properties.is_buyable, saved[0].properties.earn_apr), (34, true, Some(4.68)));
            assert_eq!(provider.requested_paths(), vec!["/v1/assets".to_string()]);
        });
    }

    #[test]
    fn test_sync_missing_assets_returns_the_ids_of_the_batch_it_saved() {
        block_on(async {
            let provider = Arc::new(TestAlienProvider::with_json(200, USDT_RESPONSE));
            let store = Arc::new(MemoryAssetStore::default());
            let service = GemAssetsService::mock(provider.clone(), store.clone());

            let ids = service.sync_missing_assets(vec![ETHEREUM_USDT_ASSET_ID.clone(), ETHEREUM_USDT_ASSET_ID.clone()]).await.unwrap();

            assert_eq!(ids, vec![ETHEREUM_USDT_ASSET_ID.clone()], "a duplicated request is asked for and returned once");
            let saved = store.assets.lock().unwrap().clone();
            assert_eq!(saved.len(), 1);
            assert_eq!(saved[0].score.rank, 34, "the saved batch keeps the backend metadata");
            assert!(service.sync_missing_assets(vec![ETHEREUM_USDT_ASSET_ID.clone()]).await.unwrap().is_empty(), "a stored asset is no longer missing");
        });
    }

    #[test]
    fn test_ensure_token_asset_refuses_the_token_mirroring_the_native_coin() {
        block_on(async {
            let provider = Arc::new(TestAlienProvider::with_json(200, USDT_RESPONSE));
            let store = Arc::new(MemoryAssetStore::default());
            let service = GemAssetsService::mock(provider.clone(), store.clone());

            let mirror = AssetId::from_token(Chain::Arc, ARC_USDC_TOKEN_ID);

            let error = service.ensure_token_asset(mirror.clone()).await.unwrap_err();

            assert_eq!(
                error,
                GemServiceError::Unsupported {
                    msg: format!("{mirror} mirrors the native coin")
                }
            );
            assert!(provider.requested_paths().is_empty());
        });
    }

    #[test]
    fn test_ensure_token_asset_asks_the_node_only_after_the_backend() {
        block_on(async {
            let provider = Arc::new(TestAlienProvider::with_status(404));
            let store = Arc::new(MemoryAssetStore::default());
            let service = GemAssetsService::mock(provider.clone(), store.clone());

            let result = service.ensure_token_asset(ETHEREUM_USDT_ASSET_ID.clone()).await;

            assert!(result.is_err());
            assert!(store.assets.lock().unwrap().is_empty());
            assert_eq!(provider.requested_paths(), vec!["/v1/assets".to_string(), crate::services::node::rules::preferred_chain_node(Chain::Ethereum, None).url]);
        });
    }
}
