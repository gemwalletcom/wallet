use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId, NFTData, try_in_order};

#[async_trait]
pub trait NFTProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chains(&self) -> &'static [NFTChain];
    async fn get_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>>;
    async fn get_collection(&self, collection: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>>;
    async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>>;
    async fn get_nft_assets(&self, chain: Chain, address: String) -> Result<Vec<NFTAsset>, Box<dyn Error + Send + Sync>> {
        let ids = self.get_assets(chain, address).await?;
        let mut assets = Vec::with_capacity(ids.len());
        for id in ids {
            if let Ok(asset) = self.get_asset(id).await {
                assets.push(asset);
            }
        }
        Ok(assets)
    }
    async fn get_nft_data(&self, chain: Chain, address: String) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let assets = self.get_nft_assets(chain, address).await?;
        let mut by_collection: HashMap<NFTCollectionId, Vec<NFTAsset>> = HashMap::new();
        for asset in assets {
            by_collection.entry(asset.collection_id.clone()).or_default().push(asset);
        }
        let mut result = Vec::with_capacity(by_collection.len());
        for (collection_id, assets) in by_collection {
            if let Ok(collection) = self.get_collection(collection_id).await {
                result.push(NFTData { collection, assets });
            }
        }
        Ok(result)
    }
}

pub struct NFTProviders {
    providers: Vec<Arc<dyn NFTProvider>>,
}

impl NFTProviders {
    pub fn new(providers: Vec<Arc<dyn NFTProvider>>) -> Self {
        Self { providers }
    }

    fn providers_for_chain(&self, chain: Chain) -> impl Iterator<Item = &Arc<dyn NFTProvider>> {
        self.providers
            .iter()
            .filter(move |provider| provider.chains().iter().any(|nft_chain| Chain::from(*nft_chain) == chain))
    }

    async fn fetch_assets(chain: Chain, address: String, providers: impl Iterator<Item = &Arc<dyn NFTProvider>>) -> Vec<NFTAssetId> {
        let operations = providers.map(|provider| provider.get_assets(chain, address.clone())).collect::<Vec<_>>();
        match try_in_order(operations).await {
            Ok(Some(asset_ids)) => asset_ids,
            Ok(None) | Err(_) => Vec::new(),
        }
    }

    pub async fn get_assets(&self, addresses: HashMap<Chain, String>) -> Vec<NFTAssetId> {
        let futures = addresses.into_iter().map(|(chain, address)| {
            let providers = self.providers_for_chain(chain);
            async move { Self::fetch_assets(chain, address, providers).await }
        });

        futures::future::join_all(futures).await.into_iter().flatten().collect()
    }

    pub async fn get_collection(&self, collection_id: NFTCollectionId) -> Option<NFTCollection> {
        let operations = self
            .providers_for_chain(collection_id.chain)
            .map(|provider| provider.get_collection(collection_id.clone()))
            .collect::<Vec<_>>();
        try_in_order(operations).await.ok().flatten()
    }

    pub async fn get_asset(&self, asset_id: NFTAssetId) -> Option<NFTAsset> {
        let operations = self
            .providers_for_chain(asset_id.chain)
            .map(|provider| provider.get_asset(asset_id.clone()))
            .collect::<Vec<_>>();
        try_in_order(operations).await.ok().flatten()
    }

    pub async fn get_nft_data(&self, chain: Chain, address: &str) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let provider = self
            .providers_for_chain(chain)
            .next()
            .ok_or_else(|| format!("no NFT provider for chain {}", chain.as_ref()))?;
        provider.get_nft_data(chain, address.to_string()).await
    }
}
