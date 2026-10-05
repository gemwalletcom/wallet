use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{ThrottleCacher, ThrottledTask};
use chain_providers::ChainProviders;
use streamer::{ChainAddressPayload, StreamProducerQueue, consumer::MessageConsumer};

use crate::assets::repository::Repository;

pub struct FetchTokenAddressesConsumer {
    pub provider: ChainProviders,
    pub(crate) repository: Arc<dyn Repository>,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
    pub throttle: Arc<dyn ThrottleCacher>,
}

impl FetchTokenAddressesConsumer {
    pub(crate) fn new(provider: ChainProviders, repository: Arc<dyn Repository>, stream_producer: Arc<dyn StreamProducerQueue>, throttle: Arc<dyn ThrottleCacher>) -> Self {
        Self {
            provider,
            repository,
            stream_producer,
            throttle,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTokenAddressesConsumer {
    async fn should_consume(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle
            .try_start(ThrottledTask::FetchTokenAddresses {
                chain: payload.value.chain.as_ref(),
                address: &payload.value.address,
            })
            .await
    }

    async fn consume(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain_address = payload.value;
        let balances = self.provider.get_balance_assets(chain_address.chain, chain_address.address.clone()).await?;
        let update = self.repository.update_token_addresses(chain_address, balances).await?;
        self.stream_producer.publish_fetch_assets(update.unknown_asset_ids).await?;
        Ok(update.added)
    }
}
