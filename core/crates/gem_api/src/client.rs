use gem_client::{Client, ClientError, ClientExt};
use primitives::currency::Currency;
use primitives::{
    AssetBasic, AssetFull, AssetId, AssetPrice, AssetPrices, AssetPricesRequest, Chain, ChartPeriod, Charts, ConfigResponse, FiatAssets, FiatQuoteType, SearchResponse,
};

use crate::target::GemApiTarget;

#[derive(Debug, Clone)]
pub struct GemApiClient<C: Client> {
    client: C,
}

impl<C: Client> GemApiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_config(&self) -> Result<ConfigResponse, ClientError> {
        self.client.get(GemApiTarget::GetConfig).await
    }

    pub async fn get_charts(&self, asset_id: AssetId, period: ChartPeriod) -> Result<Charts, ClientError> {
        self.client.get(GemApiTarget::GetCharts { asset_id, period }).await
    }

    pub async fn get_asset(&self, asset_id: AssetId) -> Result<AssetFull, ClientError> {
        self.client.get(GemApiTarget::GetAsset { asset_id }).await
    }

    pub async fn get_assets(&self, asset_ids: Vec<AssetId>, currency: Option<String>) -> Result<Vec<AssetBasic>, ClientError> {
        let identifiers = asset_ids.iter().map(AssetId::to_string).collect::<Vec<_>>();
        self.client.post(GemApiTarget::GetAssets { currency }, &identifiers).await
    }

    pub async fn get_search_assets(&self, query: String, chains: Vec<Chain>) -> Result<Vec<AssetBasic>, ClientError> {
        self.client.get(GemApiTarget::GetSearchAssets { query, chains }).await
    }

    pub async fn get_search(&self, query: String, chains: Vec<Chain>, tags: Vec<String>) -> Result<SearchResponse, ClientError> {
        self.client.get(GemApiTarget::GetSearch { query, chains, tags }).await
    }

    pub async fn get_prices(&self, currency: Option<Currency>, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, ClientError> {
        let request = AssetPricesRequest { currency, asset_ids };
        Ok(self.client.post::<_, AssetPrices>(GemApiTarget::GetPrices, &request).await?.prices)
    }

    pub async fn get_fiat_assets(&self, quote_type: FiatQuoteType) -> Result<FiatAssets, ClientError> {
        self.client.get(GemApiTarget::GetFiatAssets { quote_type }).await
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, ClientError> {
        self.client.get(GemApiTarget::GetSwapAssets).await
    }
}
