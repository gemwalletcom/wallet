use std::collections::HashMap;
use std::error::Error;

use ::jupiter::{JupiterClient, Token};
use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::AssetId;

use crate::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProvider, PriceProviderAsset};

use super::mapper::{map_token_asset, map_token_price, to_asset_price_mapping, to_jupiter_token_id};

pub struct JupiterProvider {
    jupiter_client: JupiterClient,
}

impl JupiterProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            jupiter_client: JupiterClient::new_with_client(client),
        }
    }

    async fn verified_tokens(&self) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        self.jupiter_client.get_verified_tokens().await
    }
}

#[async_trait]
impl PriceAssetsProvider for JupiterProvider {
    fn provider(&self) -> PriceProvider {
        PriceProvider::Jupiter
    }

    async fn get_assets(&self, limit: usize) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self.verified_tokens().await?.into_iter().take(limit).map(map_token_asset).collect())
    }

    async fn get_mappings_for_asset_id(&self, asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(asset_id
            .token_id
            .clone()
            .map(|token_id| AssetPriceMapping::new(asset_id.clone(), token_id))
            .into_iter()
            .collect())
    }

    async fn get_mappings_for_price_id(&self, provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        Ok(vec![to_asset_price_mapping(provider_price_id)])
    }

    async fn get_prices(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let tokens: HashMap<String, Token> = self.verified_tokens().await?.into_iter().map(|t| (t.id.clone(), t)).collect();
        Ok(mappings
            .into_iter()
            .filter_map(|mapping| tokens.get(&to_jupiter_token_id(&mapping.provider_price_id)).map(|token| map_token_price(mapping, token)))
            .collect())
    }
}

#[cfg(all(test, feature = "price_integration_tests"))]
mod price_integration_tests {
    use std::error::Error;

    use crate::{PriceAssetsProvider, PriceProvider};

    use super::super::testkit::create_jupiter_test_provider;

    const ASSET_LIMIT: usize = 10;

    #[tokio::test]
    async fn test_jupiter_provider_basic() -> Result<(), Box<dyn Error + Send + Sync>> {
        let provider = create_jupiter_test_provider();
        assert_eq!(provider.provider(), PriceProvider::Jupiter);

        let assets = provider.get_assets(ASSET_LIMIT).await?;
        assert_eq!(assets.len(), ASSET_LIMIT);
        for asset in assets {
            assert_ne!(asset.mapping.provider_price_id, "");
        }
        Ok(())
    }
}
