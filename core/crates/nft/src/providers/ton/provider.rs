use std::collections::HashSet;
use std::error::Error;

use gem_client::Client;
use gem_ton::models::NftOffchainMetadata;
use gem_ton::rpc::client::TonClient;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

use super::mapper::{map_asset, map_assets, map_collection, map_indexed_assets, map_offchain_asset};
use crate::provider::NFTProvider;

#[async_trait::async_trait]
impl<C: Client + Send + Sync> NFTProvider for TonClient<C> {
    fn name(&self) -> &'static str {
        "Ton"
    }

    fn chains(&self) -> &'static [NFTChain] {
        &[NFTChain::Ton]
    }

    async fn get_assets(&self, _chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_items_by_owner(&address).await?;
        Ok(map_assets(&response))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_collection(&collection_id.contract_address).await?;
        map_collection(response, collection_id).ok_or_else(|| "Collection not found".into())
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_item(&asset_id.token_id).await?;
        if let Some(asset) = map_asset(&response, asset_id.clone()) {
            return Ok(asset);
        }

        let uri = response
            .nft_items
            .first()
            .and_then(|item| item.content.as_ref())
            .and_then(|content| content.uri.as_deref())
            .ok_or("Asset not found")?;
        let metadata: NftOffchainMetadata = self.client.get_url(uri).await?;
        map_offchain_asset(metadata, asset_id).ok_or_else(|| "Asset not found".into())
    }

    async fn get_nft_assets(&self, _chain: Chain, address: String) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let response = self.get_nft_items_by_owner(&address).await?;
        let asset_ids = map_assets(&response);
        let mut assets = map_indexed_assets(&response);
        let existing_ids: HashSet<NFTAssetId> = assets.iter().map(|asset| asset.id.clone()).collect();

        for asset_id in asset_ids.into_iter().filter(|asset_id| !existing_ids.contains(asset_id)) {
            if let Ok(asset) = self.get_asset(asset_id).await {
                assets.push(asset);
            }
        }

        Ok(assets)
    }
}
