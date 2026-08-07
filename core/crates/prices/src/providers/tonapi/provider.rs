use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::AssetId;
use tokio::time::sleep;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset};

use super::client::TonApiClient;
use super::mapper::{map_price, mapping_for_asset_id, mapping_for_price_id, mapping_for_stonfi_asset};
use super::stonfi_client::StonfiClient;

const RATES_PER_REQUEST: usize = 100;
const REQUEST_INTERVAL: Duration = Duration::from_secs(1);

pub struct TonApiProvider {
    client: TonApiClient,
    stonfi_client: StonfiClient,
}

impl TonApiProvider {
    pub fn new(client: ReqwestClient, stonfi_client: ReqwestClient, api_key: &str) -> Self {
        Self {
            client: TonApiClient::new(client, api_key),
            stonfi_client: StonfiClient::new(stonfi_client),
        }
    }

    async fn prices_for_mappings(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        let mut prices = Vec::with_capacity(mappings.len());
        for (index, chunk) in mappings.chunks(RATES_PER_REQUEST).enumerate() {
            if index > 0 {
                sleep(REQUEST_INTERVAL).await;
            }
            let tokens = chunk.iter().map(|mapping| mapping.provider_price_id.clone()).collect::<Vec<_>>();
            let response = self.client.get_rates(&tokens).await?;
            prices.extend(chunk.iter().filter_map(|mapping| map_price(mapping.clone(), &response)));
        }
        Ok(prices)
    }
}

#[async_trait]
impl PriceAssetsProvider for TonApiProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::TonApi
    }

    async fn get_assets(&self, limit: usize) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        let mappings = self
            .stonfi_client
            .get_assets(limit.saturating_mul(5))
            .await?
            .asset_list
            .into_iter()
            .filter_map(mapping_for_stonfi_asset)
            .collect();
        Ok(self
            .prices_for_mappings(mappings)
            .await?
            .into_iter()
            .take(limit)
            .map(|price| PriceProviderAsset::with_price(price.mapping, None, Some(price.price.price), Some(price.price.price_change_percentage_24h)))
            .collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(mapping_for_asset_id(asset_id).into_iter().collect())
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(mapping_for_price_id(provider_price_id).into_iter().collect())
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        self.prices_for_mappings(mappings).await
    }
}
