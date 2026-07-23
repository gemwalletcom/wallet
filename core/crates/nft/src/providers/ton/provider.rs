use std::error::Error;

use futures::{StreamExt, stream};
use gem_client::Client;
use gem_ton::rpc::client::TonClient;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId};

use super::mapper::{asset_id_from_item, map_asset, map_assets, map_collection, map_indexed_asset, map_offchain_asset};
use crate::config::OffchainClientConfig;
use crate::offchain_client::OffchainClient;
use crate::provider::NFTProvider;

pub struct TonNftProvider<C: Client> {
    client: TonClient<C>,
    offchain_client: OffchainClient,
    concurrency: usize,
}

impl<C: Client> TonNftProvider<C> {
    pub(crate) fn new(client: TonClient<C>, config: OffchainClientConfig) -> Self {
        let concurrency = config.concurrency;
        Self {
            client,
            offchain_client: OffchainClient::new(config),
            concurrency,
        }
    }
    async fn get_offchain_asset(&self, uri: &str, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let metadata = self.offchain_client.get(uri).await?;
        map_offchain_asset(metadata, asset_id).ok_or_else(|| "Asset not found".into())
    }
}

#[async_trait::async_trait]
impl<C: Client + Send + Sync> NFTProvider for TonNftProvider<C> {
    fn name(&self) -> &'static str {
        "Ton"
    }

    fn chains(&self) -> &'static [NFTChain] {
        &[NFTChain::Ton]
    }

    async fn get_assets(&self, _chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_nft_items_by_owner(&address).await?;
        Ok(map_assets(&response))
    }

    async fn get_collection(&self, collection_id: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_nft_collection(&collection_id.contract_address).await?;
        map_collection(response, collection_id).ok_or_else(|| "Collection not found".into())
    }

    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_nft_item(&asset_id.token_id).await?;
        if let Some(asset) = map_asset(&response, asset_id.clone()) {
            return Ok(asset);
        }
        let item = response.nft_items.first().ok_or("Asset not found")?;
        let uri = item.content.as_ref().and_then(|content| content.uri.as_deref()).ok_or("Asset not found")?;
        self.get_offchain_asset(uri, asset_id).await
    }

    async fn get_nft_assets(&self, _chain: Chain, address: String) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_nft_items_by_owner(&address).await?;
        let mut assets = Vec::new();
        let mut requests = Vec::new();
        for item in &response.nft_items {
            if let Some(asset_id) = asset_id_from_item(item) {
                if let Some(asset) = map_indexed_asset(&response, item, asset_id.clone()) {
                    assets.push(asset);
                } else if let Some(uri) = item.content.as_ref().and_then(|content| content.uri.clone()) {
                    requests.push((uri, asset_id));
                }
            }
        }
        let offchain_assets = stream::iter(requests)
            .map(|(uri, asset_id)| async move { self.get_offchain_asset(&uri, asset_id).await })
            .buffer_unordered(self.concurrency)
            .filter_map(async |result| result.ok())
            .collect::<Vec<_>>()
            .await;
        assets.extend(offchain_assets);
        Ok(assets)
    }
}
