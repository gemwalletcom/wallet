pub mod config;
pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{Asset, AssetBasic, AssetFull, AssetId, AssetPrice, Chain, ConfigVersions, FiatAssets, FiatQuoteType, SearchResponse, Wallet, WalletId};

pub use model::{AssetList, GemAssetAction, GemAssetFilter};
pub use store::GemAssetStore;

use crate::api::{GemApiClient, GemApiError};
use crate::gateway::GemGateway;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemAssetsService {
    api: Arc<GemApiClient>,
    gateway: Arc<GemGateway>,
    store: Arc<dyn GemAssetStore>,
    price: Arc<GemPriceService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemAssetsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, gateway: Arc<GemGateway>, store: Arc<dyn GemAssetStore>, price: Arc<GemPriceService>, preferences: Arc<GemPreferencesService>) -> Self {
        Self {
            api,
            gateway,
            store,
            price,
            preferences,
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
        let asset = self.gateway.get_token_data(asset_id.chain, token_id).await?;
        self.store.save_assets(vec![rules::default_asset_basic(asset.clone())]).await?;
        Ok(asset)
    }

    pub async fn sync_asset(&self, asset_id: AssetId, currency: Currency) -> Result<AssetFull, GemServiceError> {
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

    pub async fn sync_assets(&self, asset_ids: Vec<AssetId>, currency: Currency) -> Result<(), GemServiceError> {
        let asset_ids = crate::services::collections::unique(asset_ids);
        if asset_ids.is_empty() {
            return Ok(());
        }
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

    pub async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let stored = self.store.get_asset_ids(asset_ids).await?;
        if stored.is_empty() {
            return Ok(());
        }
        self.store.add_missing_balances(wallet_id, stored).await
    }

    pub async fn open_wallet_asset(&self, wallet: Wallet, asset_id: AssetId) -> Result<Option<Asset>, GemServiceError> {
        if !rules::can_open(&wallet, &asset_id) {
            return Ok(None);
        }
        let asset = self.ensure_asset(asset_id.clone()).await?;
        self.add_missing_balances(wallet.id, vec![asset_id]).await?;
        Ok(Some(asset))
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        let (enabled, disabled) = rules::default_balances(&wallet);
        self.store.add_balances(wallet.id.clone(), enabled, true).await?;
        self.store.add_balances(wallet.id, disabled, false).await
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
}

impl GemAssetsService {
    pub async fn sync_availability(&self, versions: ConfigVersions) -> Result<(), GemServiceError> {
        for (list, remote_version) in rules::asset_list_versions(&versions) {
            if !rules::is_asset_list_outdated(self.preferences.get_assets_version(list).as_deref(), remote_version) {
                continue;
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
            self.preferences.set_assets_version(list, assets.version.to_string())?;
        }
        Ok(())
    }

    pub async fn search_tokens(&self, token_id: String, chains: Vec<Chain>) -> Vec<AssetBasic> {
        let lookups = chains.into_iter().map(|chain| {
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
        Ok(self.store.get_assets(vec![asset_id.clone()]).await?.into_iter().next())
    }
}

pub fn asset_action_filters(action: GemAssetAction) -> Vec<GemAssetFilter> {
    rules::asset_action_filters(action)
}

pub fn popular_asset_ids() -> Vec<AssetId> {
    rules::popular_asset_ids()
}

pub fn default_token_chain(chains: Vec<Chain>) -> Option<Chain> {
    rules::default_token_chain(&chains)
}
