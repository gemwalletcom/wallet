use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use chain_providers::ChainProviders;
use storage::Database;
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

use super::addresses::update_token_addresses;

pub struct FetchTokenAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTokenAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Database, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self { provider, database, stream_producer, cacher }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTokenAddressesConsumer {
    async fn should_process(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchTokenAddresses(payload.value.chain.as_ref(), &payload.value.address)).await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain_address = payload.value;
        let balances = self.provider.get_balance_assets(chain_address.chain, chain_address.address.clone()).await?;
        let update = self.database.run(move |client| update_token_addresses(client, chain_address, balances)).await?;
        self.stream_producer.publish_fetch_assets(update.unknown_asset_ids).await?;
        Ok(update.added)
    }
}
