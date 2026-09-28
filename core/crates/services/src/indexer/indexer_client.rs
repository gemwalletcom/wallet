use std::error::Error;

use cacher::{CacheKey, CacherClient};
use primitives::{AssetId, ChainAddress, NFTAssetId, TransactionId};
use storage::{AssetsRepository, Database};
use streamer::{ChainAddressPayload, FetchAssetAssociationsPayload, FetchListPayload, FetchPricesPayload, StreamProducer, StreamProducerQueue};

pub struct IndexerClient {
    database: Database,
    cacher: CacherClient,
    stream_producer: StreamProducer,
}

impl IndexerClient {
    pub fn new(database: Database, cacher: CacherClient, stream_producer: StreamProducer) -> Self {
        Self { database, cacher, stream_producer }
    }

    pub async fn refresh_addresses(&self, addresses: &[ChainAddress]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let cache_keys = addresses.iter().flat_map(refresh_cache_keys).collect::<Vec<_>>();
        self.cacher.delete_keys(&cache_keys).await?;
        self.stream_producer.publish_new_addresses(addresses.iter().cloned().map(ChainAddressPayload::from).collect()).await?;
        Ok(())
    }

    pub async fn refresh_asset(&self, asset_id: AssetId) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.delete(&CacheKey::FetchAssets(&asset_id.to_string()).key()).await?;
        self.stream_producer.publish_fetch_assets(vec![asset_id]).await?;
        Ok(())
    }

    pub async fn fetch_asset_associations(&self, payload: FetchAssetAssociationsPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_fetch_asset_associations(payload).await?;
        Ok(())
    }

    pub async fn fetch_asset_status(&self, asset_id: AssetId) -> Result<(), Box<dyn Error + Send + Sync>> {
        let lookup_id = asset_id.clone();
        self.database.run(move |client| client.get_asset(&lookup_id)).await?;
        self.stream_producer.publish_fetch_asset_status(asset_id).await?;
        Ok(())
    }

    pub async fn fetch_list(&self, payload: FetchListPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_fetch_list(payload).await?;
        Ok(())
    }

    pub async fn fetch_prices(&self, payload: FetchPricesPayload) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_fetch_prices(payload).await?;
        Ok(())
    }

    pub async fn fetch_nft_asset(&self, asset_id: NFTAssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.stream_producer.publish_fetch_nft_asset(asset_id).await
    }

    pub async fn refresh_nft_asset(&self, asset_id: NFTAssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.delete(&CacheKey::FetchNftAsset(&asset_id.to_string()).key()).await?;
        self.fetch_nft_asset(asset_id).await
    }

    pub async fn refresh_transaction(&self, transaction_id: TransactionId) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.delete(&CacheKey::FetchTransaction(transaction_id.chain.as_ref(), &transaction_id.hash).key()).await?;
        self.stream_producer.publish_fetch_transactions(vec![transaction_id.into()]).await?;
        Ok(())
    }
}

fn refresh_cache_keys(address: &ChainAddress) -> [String; 4] {
    let chain = address.chain.as_ref();
    [
        CacheKey::FetchCoinAddresses(chain, &address.address).key(),
        CacheKey::FetchTokenAddresses(chain, &address.address).key(),
        CacheKey::FetchNftAssetsAddresses(chain, &address.address).key(),
        CacheKey::FetchAddressTransactions(chain, &address.address).key(),
    ]
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, ChainAddress};

    use super::refresh_cache_keys;

    #[test]
    fn test_refresh_cache_keys() {
        let address = ChainAddress::new(Chain::Ethereum, "0x123".to_string());

        assert_eq!(
            refresh_cache_keys(&address),
            [
                "fetch:coin_addresses:ethereum:0x123",
                "fetch:token_addresses:ethereum:0x123",
                "fetch:nft_assets_addresses:ethereum:0x123",
                "fetch:address_transactions:ethereum:0x123",
            ]
        );
    }
}
