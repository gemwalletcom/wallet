use std::collections::HashSet;

use async_trait::async_trait;
use nft::map_nft_data;
use primitives::{Chain, ChainAddress, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData};
use storage::{Database, DatabaseClient, DatabaseError, NftCollectionFilter, NftRepository, WalletsRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn wallet_subscriptions(&self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError>;
    async fn address_asset_ids(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, DatabaseError>;
    async fn asset_with_collection(&self, asset_id: String) -> Result<(NFTAsset, NFTCollection), DatabaseError>;
    async fn asset(&self, asset_id: String) -> Result<NFTAsset, DatabaseError>;
    async fn collection(&self, collection_id: String) -> Result<NFTCollection, DatabaseError>;
    async fn nfts(&self, asset_ids: Vec<String>) -> Result<Vec<NFTData>, DatabaseError>;
    async fn collection_ids(&self, identifiers: Vec<String>) -> Result<Vec<NFTCollectionId>, DatabaseError>;
    async fn asset_ids(&self, identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError>;
    async fn upsert_collection(&self, collection: NFTCollection) -> Result<(), DatabaseError>;
    async fn upsert_asset(&self, collection_id: NFTCollectionId, asset: NFTAsset) -> Result<(), DatabaseError>;
    async fn add_collections(&self, collections: Vec<NFTCollection>) -> Result<usize, DatabaseError>;
    async fn add_assets(&self, assets: Vec<NFTAsset>) -> Result<usize, DatabaseError>;
    async fn set_asset_associations(&self, associations: Vec<(String, Vec<Chain>, Vec<NFTAssetId>)>) -> Result<(), DatabaseError>;
    async fn add_report(&self, device_id: String, collection_id: String, asset_id: Option<String>, reason: Option<String>) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn wallet_subscriptions(&self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, DatabaseError> {
        self.database.run(move |client| client.get_subscriptions_by_wallet_id(device_id, wallet_id)).await
    }

    async fn address_asset_ids(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, DatabaseError> {
        self.database.run(move |client| client.get_nft_asset_ids_for_address(chain, &address)).await
    }

    async fn asset_with_collection(&self, asset_id: String) -> Result<(NFTAsset, NFTCollection), DatabaseError> {
        self.database
            .run(move |client| {
                let asset = nft_asset(client, &asset_id)?;
                let collection = client.get_nft_collection(&asset.collection_id.to_string())?;
                Ok((asset, collection))
            })
            .await
    }

    async fn asset(&self, asset_id: String) -> Result<NFTAsset, DatabaseError> {
        self.database.run(move |client| nft_asset(client, &asset_id)).await
    }

    async fn collection(&self, collection_id: String) -> Result<NFTCollection, DatabaseError> {
        self.database.run(move |client| client.get_nft_collection(&collection_id)).await
    }

    async fn nfts(&self, asset_ids: Vec<String>) -> Result<Vec<NFTData>, DatabaseError> {
        self.database
            .run(move |client| {
                let assets = client.get_nft_assets(asset_ids)?;
                let collection_ids = assets.iter().map(|asset| asset.collection_id.to_string()).collect::<HashSet<_>>().into_iter().collect();
                let collections = client.get_nft_collections(vec![NftCollectionFilter::Identifiers(collection_ids)])?;
                Ok(map_nft_data(assets, collections))
            })
            .await
    }

    async fn collection_ids(&self, identifiers: Vec<String>) -> Result<Vec<NFTCollectionId>, DatabaseError> {
        self.database.run(move |client| client.get_nft_collection_ids(identifiers)).await
    }

    async fn asset_ids(&self, identifiers: Vec<String>) -> Result<Vec<NFTAssetId>, DatabaseError> {
        self.database.run(move |client| client.get_nft_asset_ids(identifiers)).await
    }

    async fn upsert_collection(&self, collection: NFTCollection) -> Result<(), DatabaseError> {
        self.database.run(move |client| client.upsert_nft_collection(collection)).await
    }

    async fn upsert_asset(&self, collection_id: NFTCollectionId, asset: NFTAsset) -> Result<(), DatabaseError> {
        self.database.run(move |client| client.upsert_nft_asset(&collection_id, asset)).await
    }

    async fn add_collections(&self, collections: Vec<NFTCollection>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_nft_collections(collections)).await
    }

    async fn add_assets(&self, assets: Vec<NFTAsset>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_nft_assets(assets)).await
    }

    async fn set_asset_associations(&self, associations: Vec<(String, Vec<Chain>, Vec<NFTAssetId>)>) -> Result<(), DatabaseError> {
        self.database
            .run(move |client| {
                for (address, chains, owned) in associations {
                    client.set_nft_asset_associations(&address, chains, owned)?;
                }
                Ok(())
            })
            .await
    }

    async fn add_report(&self, device_id: String, collection_id: String, asset_id: Option<String>, reason: Option<String>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_nft_report(&device_id, &collection_id, asset_id, reason)).await
    }
}

fn nft_asset(client: &mut DatabaseClient, asset_id: &str) -> Result<NFTAsset, DatabaseError> {
    client.get_nft_assets(vec![asset_id.to_string()])?.into_iter().next().ok_or_else(|| DatabaseError::not_found("NftAsset", asset_id))
}
