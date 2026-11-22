use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainPerpetual;
use gem_client::Client;
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_reqwest::ReqwestClient;
use primitives::{AssetId, Chain};

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset};

use super::mapper::{map_perpetual_prices, map_spot_prices, price_mapping};

pub struct HypercoreProvider<C: Client = ReqwestClient> {
    client: HyperCoreClient<C>,
}

impl<C: Client> HypercoreProvider<C> {
    pub fn new(client: HyperCoreClient<C>) -> Self {
        Self { client }
    }

    async fn prices(&self) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        let (spot, perpetuals) = futures::try_join!(self.client.get_spot_meta_and_asset_contexts(), self.client.get_perpetuals_data())?;
        Ok(map_spot_prices(spot).into_iter().chain(map_perpetual_prices(perpetuals)).collect())
    }
}

#[async_trait]
impl<C: Client + 'static> PriceAssetsProvider for HypercoreProvider<C> {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Hypercore
    }

    async fn get_assets(&self, limit: usize) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .prices()
            .await?
            .into_iter()
            .take(limit)
            .map(|price| PriceProviderAsset::with_price(price.mapping, price.market, Some(price.price), price.price_change_percentage_24h))
            .collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(price_mapping(asset_id).into_iter().collect())
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(price_mapping(&AssetId::from_token(Chain::HyperCore, provider_price_id)).into_iter().collect())
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let prices: HashMap<String, AssetPriceFull> = self.prices().await?.into_iter().map(|price| (price.mapping.provider_price_id.clone(), price)).collect();
        Ok(mappings
            .into_iter()
            .filter_map(|mapping| prices.get(&mapping.provider_price_id).map(|price| AssetPriceFull { mapping, ..price.clone() }))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use primitives::asset_constants::{HYPERCORE_SPOT_UBTC_ASSET_ID, HYPERCORE_SPOT_UBTC_TOKEN_ID};
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_get_prices() {
        let client = HyperCoreClient::mock_with_responses_by_request(
            vec![
                (json!({"type": "spotMetaAndAssetCtxs"}), include_bytes!("../../../testdata/hypercore/spot_meta_and_asset_ctxs.json").to_vec()),
                (json!({"type": "metaAndAssetCtxs"}), include_bytes!("../../../testdata/hypercore/meta_and_asset_ctxs.json").to_vec()),
            ],
            Arc::new(Mutex::new(Vec::new())),
        );
        let provider = HypercoreProvider::new(client);
        let ubtc = AssetPriceMapping::new(HYPERCORE_SPOT_UBTC_ASSET_ID.clone(), HYPERCORE_SPOT_UBTC_TOKEN_ID.to_string());
        let btc = AssetPriceMapping::new(AssetId::from_token(Chain::HyperCore, "perpetual::BTC"), "perpetual::BTC".to_string());
        let doge = AssetPriceMapping::new(AssetId::from_token(Chain::HyperCore, "perpetual::DOGE"), "perpetual::DOGE".to_string());

        let prices: Vec<(AssetPriceMapping, f64, Option<f64>)> = provider
            .get_prices(vec![ubtc.clone(), btc.clone(), doge])
            .await
            .unwrap()
            .into_iter()
            .map(|price| (price.mapping, price.price, price.price_change_percentage_24h))
            .collect();

        assert_eq!(prices, vec![(ubtc, 65341.5, Some(0.30317450571042615)), (btc, 65418.5, Some(0.32281315176052017))]);
        assert!(provider.get_prices(vec![]).await.unwrap().is_empty());
    }
}

#[cfg(all(test, feature = "price_integration_tests"))]
mod price_integration_tests {
    use primitives::asset_constants::HYPERCORE_SPOT_UBTC_ASSET_ID;

    use super::super::testkit::create_hypercore_test_provider;
    use super::*;

    #[tokio::test]
    async fn test_hypercore_provider_get_prices() -> Result<(), Box<dyn Error + Send + Sync>> {
        let provider = create_hypercore_test_provider();
        let asset_ids = [
            HYPERCORE_SPOT_UBTC_ASSET_ID.clone(),
            AssetId::from_token(Chain::HyperCore, "perpetual::BTC"),
            AssetId::from_token(Chain::HyperCore, "perpetual::xyz:GOLD"),
        ];
        let mappings = asset_ids.iter().filter_map(price_mapping).collect();

        let prices = provider.get_prices(mappings).await?;

        assert_eq!(prices.iter().map(|price| price.mapping.asset_id.clone()).collect::<Vec<_>>(), asset_ids.to_vec());
        assert!(prices.iter().all(|price| price.price > 0.0));
        Ok(())
    }
}
