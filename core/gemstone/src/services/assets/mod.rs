pub mod error;
pub mod rules;
pub mod store;

use primitives::WalletId;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetBasic, AssetFull, AssetId, AssetPrice, Chain, FiatAssets, FiatQuoteType, SearchResponse};

pub use error::GemAssetError;
pub use store::GemAssetStore;

use crate::api::{GemApiClient, GemApiError};
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemAssetsService {
    api: Arc<GemApiClient>,
    store: Arc<dyn GemAssetStore>,
    price: Arc<GemPriceService>,
}

#[uniffi::export]
impl GemAssetsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, store: Arc<dyn GemAssetStore>, price: Arc<GemPriceService>) -> Self {
        Self { api, store, price }
    }

    pub async fn sync_asset(&self, asset_id: AssetId, currency: Currency) -> Result<AssetFull, GemAssetError> {
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

    pub async fn prefetch_assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemAssetError> {
        let existing = self.store.get_asset_ids(asset_ids.clone()).await?;
        let missing = rules::missing_asset_ids(asset_ids, existing);
        if missing.is_empty() {
            return Ok(vec![]);
        }
        let assets = self.get_assets(missing, None).await?;
        self.store.save_assets(assets.clone()).await?;
        Ok(assets.into_iter().map(|asset| asset.asset.id).collect())
    }

    pub async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemAssetError> {
        self.store.add_missing_balances(wallet_id, asset_ids).await
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
