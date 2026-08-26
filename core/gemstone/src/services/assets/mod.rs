use std::sync::Arc;

use primitives::{AssetBasic, AssetFull, AssetId, Chain, FiatAssets, FiatQuoteType, SearchResponse};

use crate::api::{GemApiClient, GemApiError};

#[derive(Debug, uniffi::Object)]
pub struct GemAssetsService {
    api: Arc<GemApiClient>,
}

#[uniffi::export]
impl GemAssetsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>) -> Self {
        Self { api }
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
