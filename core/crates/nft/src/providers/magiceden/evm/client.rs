use std::error::Error;

use gem_client::{Client, ClientExt};
use primitives::Chain;

use super::model::{CollectionDetail, CollectionsResponse, TokenDetailResponse, TokensResponse};

pub struct MagicEdenEvmClient<C: Client> {
    client: C,
}

impl<C: Client> MagicEdenEvmClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    fn chain_id(chain: Chain) -> Result<&'static str, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::SmartChain => Ok("bsc"),
            _ => Err(format!("Unsupported EVM chain for MagicEden: {:?}", chain).into()),
        }
    }

    pub async fn get_nfts_by_wallet(&self, chain: Chain, wallet_address: &str) -> Result<TokensResponse, Box<dyn Error + Send + Sync>> {
        let chain_id = Self::chain_id(chain)?;
        let response: TokensResponse = self
            .client
            .get_with_query(
                "/v4/evm-public/assets/user-assets",
                &[("chain".to_string(), chain_id.to_string()), ("walletAddresses[]".to_string(), wallet_address.to_string())],
            )
            .await?;

        Ok(response)
    }

    pub async fn fetch_collection_detail(&self, chain: Chain, collection_id: &str) -> Result<CollectionDetail, Box<dyn Error + Send + Sync>> {
        let chain_id = Self::chain_id(chain)?;
        let body = serde_json::json!({"chain": chain_id, "collectionIds": [collection_id.to_lowercase()]});
        let response: CollectionsResponse = self.client.post("/v4/evm-public/collections", &body).await?;
        response.collections.into_iter().next().ok_or_else(|| "Collection not found".into())
    }

    pub async fn get_token(&self, chain: Chain, collection_id: &str, token_id: &str) -> Result<TokenDetailResponse, Box<dyn Error + Send + Sync>> {
        let chain_id = Self::chain_id(chain)?;
        let collection_id_lower = collection_id.to_lowercase();
        let asset_id = format!("{}:{}", collection_id_lower, token_id);
        let response: TokensResponse = self
            .client
            .get_with_query(
                "/v4/evm-public/assets/collection-assets",
                &[
                    ("chain".to_string(), chain_id.to_string()),
                    ("collectionId".to_string(), collection_id_lower),
                    ("assetIds[]".to_string(), asset_id),
                ],
            )
            .await?;

        let token = response.assets.into_iter().next().ok_or("Token not found")?.asset;
        Ok(TokenDetailResponse { token })
    }
}
