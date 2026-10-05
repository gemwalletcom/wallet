use std::collections::{HashMap, HashSet};
use std::error::Error;

use nft::{NFTProviderClient, map_nft_data};
use primitives::nft::NFTAssetData;
use primitives::{AssetId, Chain, ImageFormatter, NFTAsset, NFTAssetId, NFTCollection, NFTCollectionId, NFTData};
use storage::{Database, DatabaseClient, DatabaseError, NftCollectionFilter, NftRepository, WalletsRepository};

pub struct NFTClient {
    database: Database,
    provider_client: NFTProviderClient,
    assets_url: String,
}

impl NFTClient {
    pub fn new(database: Database, provider_client: NFTProviderClient, assets_url: String) -> Self {
        Self { database, provider_client, assets_url }
    }

    pub async fn update_collection(&self, collection_id: NFTCollectionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let collection = self.provider_client.get_nft_collection(collection_id).await?;
        self.upsert_collection(collection).await?;
        Ok(true)
    }

    pub async fn refresh_asset(&self, asset_id: NFTAssetId) -> Result<(), Box<dyn Error + Send + Sync>> {
        let collection_id = asset_id.get_collection_id();
        let collection = self.provider_client.get_nft_collection(collection_id.clone()).await?;
        self.upsert_collection(collection).await?;
        let asset = self.provider_client.get_nft_asset(asset_id).await?;
        self.database.run(move |client| client.upsert_nft_asset(&collection_id, asset)).await?;
        Ok(())
    }

    pub async fn get_wallet_assets(&self, device_id: i32, wallet_id: i32) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.run(move |client| client.get_subscriptions_by_wallet_id(device_id, wallet_id)).await?;

        let mut asset_ids: HashSet<NFTAssetId> = HashSet::new();
        for subscription in subscriptions {
            let chain = subscription.chain;
            let ids = match self.provider_client.get_nft_asset_ids(chain, &subscription.address).await {
                Ok(ids) => ids,
                Err(_) => self.database.run(move |client| client.get_nft_asset_ids_for_address(chain, &subscription.address)).await?,
            };
            asset_ids.extend(ids);
        }

        self.preload(asset_ids.into_iter().collect()).await
    }

    pub async fn get_nft_asset_data(&self, asset_id: NFTAssetId) -> Result<NFTAssetData, Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.to_string();
        let (asset, collection) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let asset = Self::load_nft_asset_with(client, &asset_id)?;
                let collection = client.get_nft_collection(&asset.collection_id.to_string())?;
                Ok((asset, collection))
            })
            .await?;

        Ok(NFTAssetData {
            collection: self.with_urls_collection(collection),
            asset: self.with_urls_asset(asset),
        })
    }

    fn with_urls_asset(&self, asset: NFTAsset) -> NFTAsset {
        let id = asset.id.to_string();
        let preview_url = ImageFormatter::get_nft_asset_url(&self.assets_url, &id);
        let resource_url = ImageFormatter::get_nft_asset_resource_url(&self.assets_url, &id);
        asset.with_urls(preview_url, resource_url)
    }

    fn with_urls_collection(&self, collection: NFTCollection) -> NFTCollection {
        let preview_url = ImageFormatter::get_nft_collection_url(&self.assets_url, &collection.id.to_string());
        collection.with_preview_url(preview_url)
    }

    fn with_urls_data(&self, data: NFTData) -> NFTData {
        NFTData {
            collection: self.with_urls_collection(data.collection),
            assets: data.assets.into_iter().map(|a| self.with_urls_asset(a)).collect(),
        }
    }

    async fn upsert_collection(&self, collection: NFTCollection) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| client.upsert_nft_collection(collection)).await?)
    }

    async fn preload(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let collection_ids: HashSet<NFTCollectionId> = assets.iter().map(NFTAssetId::get_collection_id).collect();
        self.preload_collections(collection_ids.into_iter().collect()).await?;
        self.preload_assets(&assets).await?;
        self.get_nfts(assets).await
    }

    async fn preload_collections(&self, collection_ids: Vec<NFTCollectionId>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let identifiers: Vec<String> = collection_ids.iter().map(ToString::to_string).collect();
        let existing: HashSet<NFTCollectionId> = self.database.run(move |client| client.get_nft_collection_ids(identifiers)).await?.into_iter().collect();

        let mut new_collections: Vec<NFTCollection> = Vec::new();
        for id in collection_ids.into_iter().filter(|id| !existing.contains(id)) {
            if let Ok(collection) = self.provider_client.get_nft_collection(id).await {
                new_collections.push(collection);
            }
        }

        if new_collections.is_empty() {
            return Ok(());
        }
        self.database.run(move |client| client.add_nft_collections(new_collections)).await?;
        Ok(())
    }

    async fn preload_assets(&self, asset_ids: &[NFTAssetId]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let identifiers: Vec<String> = asset_ids.iter().map(ToString::to_string).collect();
        let existing: HashSet<NFTAssetId> = self.database.run(move |client| client.get_nft_asset_ids(identifiers)).await?.into_iter().collect();

        let mut new_assets: Vec<NFTAsset> = Vec::new();
        for id in asset_ids.iter().filter(|id| !existing.contains(id)).cloned() {
            if let Ok(asset) = self.provider_client.get_nft_asset(id).await {
                new_assets.push(asset);
            }
        }

        if new_assets.is_empty() {
            return Ok(());
        }
        self.database.run(move |client| client.add_nft_assets(new_assets)).await?;
        Ok(())
    }

    fn load_nfts(client: &mut DatabaseClient, asset_identifiers: Vec<String>) -> Result<Vec<NFTData>, DatabaseError> {
        let assets = client.get_nft_assets(asset_identifiers)?;
        let collection_ids = assets.iter().map(|asset| asset.collection_id.to_string()).collect::<HashSet<_>>().into_iter().collect();
        let collections = client.get_nft_collections(vec![NftCollectionFilter::Identifiers(collection_ids)])?;
        Ok(map_nft_data(assets, collections))
    }

    async fn get_nfts(&self, assets: Vec<NFTAssetId>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let identifiers = assets.into_iter().map(|asset| asset.to_string()).collect();
        let nfts = self.database.run(move |client| Self::load_nfts(client, identifiers)).await?;
        Ok(nfts.into_iter().map(|data| self.with_urls_data(data)).collect())
    }

    fn load_nft_asset_with(client: &mut DatabaseClient, asset_id: &str) -> Result<NFTAsset, DatabaseError> {
        client.get_nft_assets(vec![asset_id.to_string()])?.into_iter().next().ok_or_else(|| DatabaseError::not_found("NftAsset", asset_id))
    }

    pub async fn load_nft_asset(&self, asset_id: &str) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let asset_id = asset_id.to_string();
        Ok(self.database.run(move |client| Self::load_nft_asset_with(client, &asset_id)).await?)
    }

    pub async fn load_nft_collection(&self, collection_id: &str) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let collection_id = collection_id.to_string();
        Ok(self.database.run(move |client| client.get_nft_collection(&collection_id)).await?)
    }

    pub async fn update_assets_for_addresses(&self, addresses: HashMap<Chain, String>) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let mut all_asset_ids: HashSet<NFTAssetId> = HashSet::new();
        let mut owned_by_address: HashMap<String, HashSet<NFTAssetId>> = HashMap::new();
        let mut chains_by_address: HashMap<String, HashSet<Chain>> = HashMap::new();
        for (chain, address) in addresses {
            let Ok(ids) = self.provider_client.get_nft_asset_ids(chain, &address).await else {
                continue;
            };
            chains_by_address.entry(address.clone()).or_default().insert(chain);
            owned_by_address.entry(address).or_default().extend(ids.iter().cloned());
            all_asset_ids.extend(ids);
        }

        let asset_ids: Vec<NFTAssetId> = all_asset_ids.into_iter().collect();
        let collection_ids: Vec<NFTCollectionId> = asset_ids.iter().map(NFTAssetId::get_collection_id).collect::<HashSet<_>>().into_iter().collect();
        self.preload_collections(collection_ids).await?;
        self.preload_assets(&asset_ids).await?;

        let associations: Vec<(String, Vec<Chain>, Vec<NFTAssetId>)> = owned_by_address
            .into_iter()
            .map(|(address, owned)| {
                let chains: Vec<Chain> = chains_by_address.remove(&address).unwrap_or_default().into_iter().collect();
                (address, chains, owned.into_iter().collect())
            })
            .collect();
        self.database
            .run(move |client| -> Result<(), DatabaseError> {
                for (address, chains, owned) in associations {
                    client.set_nft_asset_associations(&address, chains, owned)?;
                }
                Ok(())
            })
            .await?;

        self.get_nfts(asset_ids).await
    }

    pub async fn report_nft(&self, device_id: &str, collection_id: String, asset_id: Option<AssetId>, reason: Option<String>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        self.database.run(move |client| client.add_nft_report(&device_id, &collection_id, asset_id.map(|id| id.to_string()), reason)).await?;
        Ok(true)
    }
}
