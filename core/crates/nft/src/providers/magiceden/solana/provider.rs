use std::error::Error;

use gem_client::Client;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

use super::client::MagicEdenSolanaClient;
use super::mapper::{map_asset, map_assets, map_collection};
use crate::provider::NFTProvider;

#[async_trait::async_trait]
impl<C: Client + 'static> NFTProvider for MagicEdenSolanaClient<C> {
    fn name(&self) -> &'static str {
        "MagicEdenSolana"
    }

    fn chains(&self) -> &[NFTChain] {
        &[NFTChain::Solana]
    }

    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nfts_by_account(&address).await?;
        Ok(map_assets(response, chain))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection = self.get_collection_id(&collection_id.contract_address).await?;
        Ok(map_collection(collection, collection_id))
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let nft = self.get_asset_id(&asset_id.token_id).await?;
        Ok(map_asset(nft.clone(), asset_id, nft.owner).ok_or("Asset not found")?)
    }
}

#[cfg(all(test, feature = "nft_integration_tests"))]
mod nft_integration_tests {
    use crate::NFTProvider;
    use crate::testkit::*;
    use primitives::{Chain, NFTAssetId, NFTCollectionId};

    #[tokio::test]
    async fn test_magiceden_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_solana_test_client();

        let assets = client.get_assets(Chain::Solana, TEST_SOLANA_ADDRESS.to_string()).await?;

        println!("Found {} MagicEden assets", assets.len());
        assert!(!assets.is_empty());

        if let Some(asset_id) = assets.first() {
            assert_eq!(asset_id.chain, Chain::Solana);
            assert!(!asset_id.token_id.is_empty());
            println!("Sample MagicEden asset: {:?}", asset_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_magiceden_get_collection() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_solana_test_client();

        let collection_id = NFTCollectionId::new(Chain::Solana, TEST_SOLANA_COLLECTION);
        let collection = client.get_collection(collection_id).await?;

        println!("MagicEden collection: {:?}", collection);
        assert_eq!(collection.chain, Chain::Solana);
        assert!(!collection.name.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_magiceden_get_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_magiceden_solana_test_client();

        let asset_id = NFTAssetId::new(Chain::Solana, TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID);

        let asset = client.get_asset(asset_id).await?;
        println!("MagicEden asset: {:?}", asset);

        assert_eq!(asset.id, NFTAssetId::new(Chain::Solana, TEST_SOLANA_COLLECTION_POOKS, TEST_SOLANA_TOKEN_ID));
        assert_eq!(asset.chain, Chain::Solana);
        assert!(!asset.name.is_empty());
        assert!(!asset.attributes.is_empty());
        assert_eq!(asset.token_id, TEST_SOLANA_TOKEN_ID);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use gem_client::{ClientError, testkit::MockClient};
    use primitives::VerificationStatus;

    use super::*;

    const OWNER: &str = "FJ1n8eKmh17NK6ZEjG8Q842TBaqabJ8k6MS1N21ps2rN";
    const MINT: &str = "W1iLZMsRanht4urj4Vra2JqgeKoLkP5LaPerzpsu8iw";

    #[tokio::test]
    async fn test_an_nft_without_collection_is_its_own_unverified_collection() {
        let tokens: Vec<serde_json::Value> = serde_json::from_str(include_str!("../../../../testdata/magiceden/assets_uncollected.json")).unwrap();
        let client = MagicEdenSolanaClient::new(MockClient::new().with_get(move |path| match path {
            "/v2/wallets/FJ1n8eKmh17NK6ZEjG8Q842TBaqabJ8k6MS1N21ps2rN/tokens" => Ok(serde_json::to_vec(&tokens[..1]).unwrap()),
            "/collections/W1iLZMsRanht4urj4Vra2JqgeKoLkP5LaPerzpsu8iw" => Ok(include_bytes!("../../../../testdata/magiceden/collection_uncollected.json").to_vec()),
            "/v2/tokens/W1iLZMsRanht4urj4Vra2JqgeKoLkP5LaPerzpsu8iw" => Ok(serde_json::to_vec(&tokens[0]).unwrap()),
            _ => Err(ClientError::Http { status: 404, body: vec![] }),
        }));

        let data = client.get_nft_data(Chain::Solana, OWNER.to_string()).await.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].collection.id, NFTCollectionId::new(Chain::Solana, MINT));
        assert_eq!(data[0].collection.status, VerificationStatus::Unverified);
        assert_eq!(data[0].assets[0].id, NFTAssetId::new(Chain::Solana, MINT, MINT));

        let asset = client.get_asset(NFTAssetId::new(Chain::Solana, MINT, MINT)).await.unwrap();
        assert_eq!(asset.collection_id, data[0].collection.id);
        assert_eq!(asset.name, "NFT Coins");
    }
}
