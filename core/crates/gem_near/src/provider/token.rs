use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainToken;
use gem_client::Client;
use primitives::Asset;

use super::token_mapper::map_token_data;
use crate::{is_valid_address, models::FungibleTokenMetadata, rpc::NearProvider};

#[async_trait]
impl<C: Client + Clone> ChainToken for NearProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        let metadata: FungibleTokenMetadata = self.call_function(&token_id, "ft_metadata", &serde_json::json!({})).await?;
        Ok(map_token_data(&token_id, metadata))
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        is_valid_address(token_id)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_near_test_client;
    use primitives::{AssetType, asset_constants::NEAR_USDT_TOKEN_ID};

    #[tokio::test]
    async fn test_near_get_token_data() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_near_test_client();
        let asset = client.get_token_data(NEAR_USDT_TOKEN_ID.to_string()).await?;

        assert_eq!(asset.token_id().as_deref(), Some(NEAR_USDT_TOKEN_ID));
        assert_eq!(asset.symbol, "USDt");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.asset_type, AssetType::TOKEN);
        Ok(())
    }
}
