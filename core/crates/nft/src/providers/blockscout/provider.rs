use std::error::Error;

use blockscout::Client as BlockscoutClient;
use gem_client::Client;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

use super::mapper::{map_asset, map_assets, map_collection};
use crate::provider::NFTProvider;

const ACCOUNT_NFTS_LIMIT: usize = 100;

pub struct BlockscoutProvider<C: Client> {
    client: BlockscoutClient<C>,
    chain: NFTChain,
}

impl<C: Client> BlockscoutProvider<C> {
    pub fn new(client: BlockscoutClient<C>, chain: NFTChain) -> Self {
        Self { client, chain }
    }
}

#[async_trait::async_trait]
impl<C: Client + 'static> NFTProvider for BlockscoutProvider<C> {
    fn name(&self) -> &'static str {
        "Blockscout"
    }

    fn chains(&self) -> &[NFTChain] {
        std::slice::from_ref(&self.chain)
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(map_assets(self.client.get_address_nfts(&address, ACCOUNT_NFTS_LIMIT).await?, chain))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let token = self.client.get_token(&collection_id.contract_address).await?;
        Ok(map_collection(token, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let instance = self.client.get_nft_instance(&asset_id.contract_address, &asset_id.token_id).await?;
        map_asset(instance, asset_id).ok_or_else(|| "Asset not found".into())
    }
}

#[cfg(all(test, feature = "nft_integration_tests"))]
mod nft_integration_tests {
    use std::error::Error;

    use primitives::{Chain, NFTAssetId, NFTCollectionId};

    use crate::NFTProvider;
    use crate::testkit::{TEST_ARC_ADDRESS, TEST_ARC_COLLECTION, create_blockscout_test_client};

    #[tokio::test]
    async fn test_blockscout_provider() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_blockscout_test_client();
        let assets = client.get_assets(Chain::Arc, TEST_ARC_ADDRESS.to_string()).await?;
        assert!(!assets.is_empty());

        let collection = client.get_collection(NFTCollectionId::new(Chain::Arc, TEST_ARC_COLLECTION)).await?;
        assert_eq!(collection.name, "The Arc Begins");

        let asset = client.get_asset(NFTAssetId::new(Chain::Arc, TEST_ARC_COLLECTION, "1")).await?;
        assert_eq!(asset.name, "The Arc Begins");
        assert!(!asset.attributes.is_empty());
        Ok(())
    }
}
