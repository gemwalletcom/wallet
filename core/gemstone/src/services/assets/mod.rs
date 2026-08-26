pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetBasic, AssetFull, AssetId, AssetPrice, Chain, ConfigVersions, FiatAssets, FiatQuoteType, SearchResponse, Wallet, WalletId};

pub use model::AssetList;
pub use store::GemAssetStore;

use crate::api::{GemApiClient, GemApiError};
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemAssetsService {
    api: Arc<GemApiClient>,
    store: Arc<dyn GemAssetStore>,
    price: Arc<GemPriceService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemAssetsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, store: Arc<dyn GemAssetStore>, price: Arc<GemPriceService>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { api, store, price, preferences }
    }

    pub async fn sync_availability(&self, versions: ConfigVersions) -> Result<(), GemServiceError> {
        let lists = [
            (AssetList::Buy, versions.fiat_on_ramp_assets),
            (AssetList::Sell, versions.fiat_off_ramp_assets),
            (AssetList::Swap, versions.swap_assets),
        ];
        for (list, remote_version) in lists {
            if self.preferences.get_assets_version(list)? == Some(remote_version.to_string()) {
                continue;
            }
            let assets = match list {
                AssetList::Buy => self.get_fiat_assets(FiatQuoteType::Buy).await?,
                AssetList::Sell => self.get_fiat_assets(FiatQuoteType::Sell).await?,
                AssetList::Swap => self.get_swap_assets().await?,
            };
            let asset_ids: Vec<AssetId> = assets.asset_ids.iter().filter_map(|id| AssetId::new(id)).collect();
            self.prefetch_assets(asset_ids.clone()).await?;
            match list {
                AssetList::Buy => self.store.set_buyable_assets(asset_ids).await?,
                AssetList::Sell => self.store.set_sellable_assets(asset_ids).await?,
                AssetList::Swap => self.store.set_swappable_assets(asset_ids).await?,
            }
            self.preferences.set_assets_version(list, assets.version.to_string())?;
        }
        Ok(())
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

    pub async fn prefetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
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
        self.store.add_missing_balances(wallet_id, asset_ids).await
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

    pub async fn get_fiat_assets(&self, quote_type: FiatQuoteType) -> Result<FiatAssets, GemApiError> {
        Ok(self.api.client.get_fiat_assets(quote_type).await?)
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, GemApiError> {
        Ok(self.api.client.get_swap_assets().await?)
    }
}
