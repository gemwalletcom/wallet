use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::{Chain, NFTAsset, NFTAssetId, NFTChain, NFTCollection, NFTCollectionId, NFTData, try_in_order};

#[async_trait]
pub trait NFTProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn chains(&self) -> &[NFTChain];
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
        self.providers.iter().filter(move |provider| provider.chains().iter().any(|nft_chain| Chain::from(*nft_chain) == chain))
    }

    pub async fn get_collection(&self, collection_id: NFTCollectionId) -> Option<NFTCollection> {
        let operations = self.providers_for_chain(collection_id.chain).map(|provider| provider.get_collection(collection_id.clone())).collect::<Vec<_>>();
        try_in_order(operations).await.ok().flatten()
    }

    pub async fn get_asset(&self, asset_id: NFTAssetId) -> Option<NFTAsset> {
        let operations = self.providers_for_chain(asset_id.chain).map(|provider| provider.get_asset(asset_id.clone())).collect::<Vec<_>>();
        try_in_order(operations).await.ok().flatten()
    }

    pub async fn get_asset_ids(&self, chain: Chain, address: &str) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        let operations = self.providers_for_chain(chain).map(|provider| provider.get_assets(chain, address.to_string())).collect::<Vec<_>>();
        try_in_order(operations).await?.ok_or_else(|| format!("no NFT provider for chain {}", chain.as_ref()).into())
    }

    pub async fn get_nft_data(&self, chain: Chain, address: &str) -> Result<Vec<NFTData>, Box<dyn Error + Send + Sync>> {
        let operations = self.providers_for_chain(chain).map(|provider| provider.get_nft_data(chain, address.to_string())).collect::<Vec<_>>();
        try_in_order(operations).await?.ok_or_else(|| format!("no NFT provider for chain {}", chain.as_ref()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubProvider {
        assets: Option<Vec<NFTAssetId>>,
    }

    #[async_trait]
    impl NFTProvider for StubProvider {
        fn name(&self) -> &'static str {
            "Stub"
        }

        fn chains(&self) -> &[NFTChain] {
            &[NFTChain::Arc]
        }

        async fn get_assets(&self, _chain: Chain, _address: String) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
            self.assets.clone().ok_or_else(|| "unavailable".into())
        }

        async fn get_collection(&self, _collection: NFTCollectionId) -> Result<NFTCollection, Box<dyn Error + Send + Sync>> {
            Err("unavailable".into())
        }

        async fn get_asset(&self, _asset_id: NFTAssetId) -> Result<NFTAsset, Box<dyn Error + Send + Sync>> {
            Err("unavailable".into())
        }
    }

    #[tokio::test]
    async fn test_get_asset_ids_falls_back_to_the_next_provider() {
        let asset_id = NFTAssetId::new(Chain::Arc, "0x1", "1");
        let providers = NFTProviders::new(vec![Arc::new(StubProvider { assets: None }), Arc::new(StubProvider { assets: Some(vec![asset_id.clone()]) })]);

        assert_eq!(providers.get_asset_ids(Chain::Arc, "0x2").await.unwrap(), vec![asset_id]);
        assert!(providers.get_asset_ids(Chain::Ethereum, "0x2").await.is_err());
    }
}
