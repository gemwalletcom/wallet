pub mod add;
pub mod config;
pub mod details;
pub mod model;
pub mod rules;
pub mod selection;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{Asset, AssetBasic, AssetFull, AssetId, AssetPrice, Chain, ConfigVersions, FiatAssets, FiatQuoteType, SearchResponse, Wallet, WalletId};

pub use add::GemAddAssetService;
pub use details::GemAssetDetailsService;
pub use model::{AssetList, GemAssetAction, GemAssetFilter, GemAssetNetworkDestination};
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
    pub fn new(
        api: Arc<GemApiClient>,
        gateway: Arc<GemGateway>,
        store: Arc<dyn GemAssetStore>,
        price: Arc<GemPriceService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
    ) -> Self {
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
        self.stored_asset(&asset_id).await?.ok_or_else(|| GemServiceError::NotFound {
            msg: format!("asset not found: {asset_id}"),
        })
    }

    pub async fn ensure_token_asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        if let Some(asset) = self.stored_asset(&asset_id).await? {
            return Ok(asset);
        }
        let Some(token_id) = asset_id.token_id.clone() else {
            return self.ensure_asset(asset_id).await;
        };
        let asset = match self.gateway.get_token_data(asset_id.chain, token_id.clone()).await {
            Ok(asset) => asset,
            Err(error) => match self.search_token_asset(&asset_id, token_id).await {
                Ok(Some(asset)) => asset,
                Ok(None) | Err(_) => return Err(error.into()),
            },
        };
        self.store.save_assets(vec![rules::default_asset_basic(asset.clone())]).await?;
        Ok(asset)
    }

    pub async fn sync_assets(&self, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let asset_ids = crate::services::collections::unique(asset_ids);
        if asset_ids.is_empty() {
            return Ok(());
        }
        let currency = self.preferences.get_currency();
        let assets = self.get_assets(asset_ids, Some(currency.to_string())).await?;
        if assets.is_empty() {
            return Ok(());
        }
        self.store.save_assets(assets.clone()).await?;
        self.price.update_prices(rules::asset_prices(&assets), currency).await
    }

    pub async fn sync_missing_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        let existing = self.store.get_asset_ids(asset_ids.clone()).await?;
        let missing = rules::missing_asset_ids(asset_ids, existing);
        if missing.is_empty() {
            return Ok(vec![]);
        }
        let assets = self.get_assets(missing, None).await?;
        self.store.save_assets(assets.clone()).await?;
        Ok(assets.into_iter().map(|asset| asset.asset.id).collect())
    }

    pub async fn open_asset(&self, asset_id: AssetId) -> Result<Option<Asset>, GemServiceError> {
        let wallet = self.session.current_wallet().await?;
        self.open_wallet_asset(wallet, asset_id).await
    }

    pub async fn open_wallet_asset(&self, wallet: Wallet, asset_id: AssetId) -> Result<Option<Asset>, GemServiceError> {
        if !rules::can_open(&wallet, &asset_id) {
            return Ok(None);
        }
        let asset = self.ensure_asset(asset_id.clone()).await?;
        self.add_missing_balances(wallet.id, vec![asset_id]).await?;
        Ok(Some(asset))
    }
}

impl GemAssetsService {
    pub async fn sync_asset(&self, asset_id: AssetId) -> Result<AssetFull, GemServiceError> {
        let currency = self.preferences.get_currency();
        let asset = self.get_asset(asset_id.clone()).await?;
        self.store.save_asset(asset.clone()).await?;
        let price = asset
            .price
            .as_ref()
            .map(|price| AssetPrice::new(asset_id.clone(), price.price, price.price_change_percentage_24h, price.updated_at));
        self.price.update_asset_price(asset_id.clone(), price, currency.clone()).await?;
        if let Some(market) = asset.market.clone() {
            self.price.update_market(asset_id, market, currency).await?;
        }
        Ok(asset)
    }

    pub(crate) async fn ensure_simulation_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        let existing = self.store.get_asset_ids(asset_ids.clone()).await?;
        let missing = rules::missing_asset_ids(asset_ids.clone(), existing);
        if missing.is_empty() {
            return self.assets(asset_ids).await;
        }
        // Simulation assets may not exist in the backend; fall back to the node.
        if let Ok(assets) = self.get_assets(missing.clone(), None).await {
            self.store.save_assets(assets).await?;
        }
        for asset_id in missing {
            self.ensure_token_asset(asset_id).await?;
        }
        self.assets(asset_ids).await
    }

    pub async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let stored = self.store.get_asset_ids(asset_ids).await?;
        if stored.is_empty() {
            return Ok(());
        }
        self.store.add_missing_balances(wallet_id, stored).await
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<Vec<AssetId>, GemServiceError> {
        let (enabled, disabled) = rules::default_balances(&wallet);
        self.store.add_balances(wallet.id.clone(), enabled.clone(), true).await?;
        self.store.add_balances(wallet.id, disabled, false).await?;
        Ok(enabled)
    }

    pub async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, GemServiceError> {
        self.store.get_assets(asset_ids).await
    }

    pub async fn get_asset(&self, asset_id: AssetId) -> Result<AssetFull, GemApiError> {
        Ok(self.api.client.get_asset(asset_id).await?)
    }

    pub async fn get_assets(&self, asset_ids: Vec<AssetId>, currency: Option<String>) -> Result<Vec<AssetBasic>, GemApiError> {
        Ok(self.api.client.get_assets(asset_ids, currency).await?)
    }

    pub async fn search_assets(&self, query: String, chains: Vec<Chain>) -> Result<Vec<AssetBasic>, GemApiError> {
        Ok(self.api.client.get_search_assets(query, chains).await?)
    }

    pub async fn search(&self, query: String, chains: Vec<Chain>, tags: Vec<String>) -> Result<SearchResponse, GemApiError> {
        Ok(self.api.client.get_search(query, chains, tags).await?)
    }

    async fn search_token_asset(&self, asset_id: &AssetId, token_id: String) -> Result<Option<Asset>, GemApiError> {
        let assets = self.api.client.get_search_assets(token_id, vec![asset_id.chain]).await?;
        Ok(assets.into_iter().map(|basic| basic.asset).find(|asset| &asset.id == asset_id))
    }

    pub async fn sync_availability(&self, versions: ConfigVersions) -> Result<(), GemServiceError> {
        let results = futures::future::join_all(
            rules::asset_list_versions(&versions)
                .into_iter()
                .map(|(list, remote_version)| self.sync_availability_list(list, remote_version)),
        )
        .await;
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
            AssetList::Swap => self.store.set_swappable_assets(asset_ids).await?,
        }
        self.preferences.set_assets_version(list, assets.version.to_string())
    }

    pub async fn search_tokens(&self, token_id: String, chains: Vec<Chain>) -> Vec<AssetBasic> {
        let lookups = chains.into_iter().filter(|chain| chain.default_asset_type().is_some()).map(|chain| {
            let token_id = token_id.clone();
            async move {
                if self.gateway.get_is_token_address(chain, token_id.clone()).await.ok()? {
                    self.gateway.get_token_data(chain, token_id).await.ok().map(rules::default_asset_basic)
                } else {
                    None
                }
            }
        });
        futures::future::join_all(lookups).await.into_iter().flatten().collect()
    }

    pub async fn sync_default_assets(&self) -> Result<(), GemServiceError> {
        let assets = rules::default_assets();
        let existing = self.store.get_asset_ids(assets.iter().map(|asset| asset.asset.id.clone()).collect()).await?;
        let missing = rules::missing_assets(assets, existing);
        if !missing.is_empty() {
            self.store.save_assets(missing).await?;
        }
        self.store.set_stakeable_assets(rules::stakeable_asset_ids()).await
    }

    pub async fn search_assets_and_tokens(&self, query: String, chains: Vec<Chain>) -> Result<Vec<AssetBasic>, GemServiceError> {
        let token_chains = rules::token_search_chains(&chains);
        let (assets, tokens) = futures::join!(self.search_assets(query.clone(), chains), self.search_tokens(query, token_chains));
        let mut assets = assets?;
        assets.extend(tokens);
        Ok(assets)
    }

    pub async fn sync_swappable_chains(&self) -> Result<(), GemServiceError> {
        self.store.set_swappable_assets(rules::swappable_chain_asset_ids()).await
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

pub fn popular_asset_ids() -> Vec<AssetId> {
    rules::popular_asset_ids()
}

pub fn default_token_chain(chains: Vec<Chain>) -> Option<Chain> {
    rules::default_token_chain(&chains)
}
