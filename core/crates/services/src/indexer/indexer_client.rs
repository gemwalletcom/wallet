use std::error::Error;
use std::sync::Arc;

use primitives::{AssetId, ChainAddress, NFTAssetId, TransactionIdRequest};
use storage::{AssetsRepository, Database};
use streamer::{ChainAddressPayload, FetchAssetAssociationsPayload, FetchListPayload, FetchPricesPayload, StreamProducer, StreamProducerQueue};

use crate::throttle_cacher::{ThrottleCacher, ThrottledTask};

pub struct IndexerClient {
    database: Database,
    throttle: Arc<dyn ThrottleCacher>,
    stream_producer: StreamProducer,
}

impl IndexerClient {
    pub fn new(database: Database, throttle: Arc<dyn ThrottleCacher>, stream_producer: StreamProducer) -> Self {
        Self { database, throttle, stream_producer }
    }

    pub async fn refresh_addresses(&self, addresses: &[ChainAddress]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let fetches = addresses.iter().flat_map(address_fetches).collect::<Vec<_>>();
        self.throttle.reset(&fetches).await?;
        self.stream_producer.publish_new_addresses(addresses.iter().cloned().map(ChainAddressPayload::from).collect()).await?;
        Ok(())
    }

    pub async fn refresh_asset(&self, asset_id: AssetId) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.throttle.reset(&[ThrottledTask::FetchAssets { asset_id: &asset_id.to_string() }]).await?;
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
        self.throttle.reset(&[ThrottledTask::FetchNftAsset { asset_id: &asset_id.to_string() }]).await?;
        self.fetch_nft_asset(asset_id).await
    }

    pub async fn refresh_transaction(&self, request: TransactionIdRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.throttle
            .reset(&[ThrottledTask::FetchTransaction {
                chain: request.chain.as_ref(),
                hash: &request.hash,
            }])
            .await?;
        self.stream_producer.publish_fetch_transactions(vec![request]).await?;
        Ok(())
    }
}

fn address_fetches(address: &ChainAddress) -> [ThrottledTask<'_>; 4] {
    let (chain, address) = (address.chain.as_ref(), address.address.as_str());
    [
        ThrottledTask::FetchCoinAddresses { chain, address },
        ThrottledTask::FetchTokenAddresses { chain, address },
        ThrottledTask::FetchNftAssetsAddresses { chain, address },
        ThrottledTask::FetchAddressTransactions { chain, address },
    ]
}

#[cfg(test)]
mod tests {
    use primitives::{Chain, ChainAddress};

    use super::address_fetches;
    use crate::throttle_cacher::ThrottledTask;

    #[test]
    fn test_address_fetches() {
        let address = ChainAddress::new(Chain::Ethereum, "0x123".to_string());
        let (chain, address_value) = ("ethereum", "0x123");

        assert_eq!(
            address_fetches(&address),
            [
                ThrottledTask::FetchCoinAddresses { chain, address: address_value },
                ThrottledTask::FetchTokenAddresses { chain, address: address_value },
                ThrottledTask::FetchNftAssetsAddresses { chain, address: address_value },
                ThrottledTask::FetchAddressTransactions { chain, address: address_value },
            ]
        );
    }
}
