use std::error::Error;

use gem_client::Client;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

use super::AlchemyClient;
use super::mapper::{map_asset, map_assets, map_collection};
use crate::provider::NFTProvider;

const PAGE_SIZE: usize = 50;

pub struct AlchemyProvider<C: Client> {
    client: AlchemyClient<C>,
    chain: NFTChain,
}

impl<C: Client> AlchemyProvider<C> {
    pub fn new(client: AlchemyClient<C>, chain: NFTChain) -> Self {
        Self { client, chain }
    }
}

#[async_trait::async_trait]
impl<C: Client + 'static> NFTProvider for AlchemyProvider<C> {
    fn name(&self) -> &'static str {
        "Alchemy"
    }

    fn chains(&self) -> &[NFTChain] {
        std::slice::from_ref(&self.chain)
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        Ok(map_assets(self.client.get_nfts_by_owner(&address, PAGE_SIZE).await?, chain))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let metadata = self.client.get_contract_metadata(&collection_id.contract_address).await?;
        Ok(map_collection(metadata, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let metadata = self.client.get_nft_metadata(&asset_id.contract_address, &asset_id.token_id).await?;
        map_asset(metadata, asset_id).ok_or_else(|| "Asset not found".into())
    }
}

#[cfg(all(test, feature = "nft_integration_tests"))]
mod nft_integration_tests {
    use std::error::Error;

    use primitives::{Chain, NFTAssetId, NFTCollectionId};

    use crate::NFTProvider;
    use crate::testkit::{TEST_BSC_ADDRESS, TEST_BSC_COLLECTION, create_alchemy_test_client};

    #[tokio::test]
    async fn test_alchemy_provider() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_alchemy_test_client();
        let assets = client.get_assets(Chain::SmartChain, TEST_BSC_ADDRESS.to_string()).await?;
        assert!(!assets.is_empty());

        let collection_id = NFTCollectionId::new(Chain::SmartChain, TEST_BSC_COLLECTION);
        let collection = client.get_collection(collection_id).await?;
        assert_eq!(collection.name, "Reefers by CoralApp");

        let asset_id = NFTAssetId::new(Chain::SmartChain, TEST_BSC_COLLECTION, "410");
        let asset = client.get_asset(asset_id).await?;
        assert_eq!(asset.name, "Reefers by CoralApp #411");
        assert!(!asset.attributes.is_empty());
        Ok(())
    }
}
