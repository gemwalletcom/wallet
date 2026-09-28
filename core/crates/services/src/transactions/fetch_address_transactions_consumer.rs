use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use chain_providers::{ChainProviders, TransactionsRequest, TransactionsResult};
use config_keys::ConfigParamKey;
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

pub struct FetchAddressTransactionsConsumer {
    pub providers: ChainProviders,
    pub producer: StreamProducer,
    pub cacher: CacherClient,
    pub config: Arc<ConfigCacher>,
}

impl FetchAddressTransactionsConsumer {
    pub fn new(providers: ChainProviders, producer: StreamProducer, cacher: CacherClient, config: Arc<ConfigCacher>) -> Self {
        Self { providers, producer, cacher, config }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchAddressTransactionsConsumer {
    async fn should_process(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchAddressTransactions(payload.value.chain.as_ref(), &payload.value.address)).await
    }
    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.value.chain;
        let limit = self.config.get_param_usize(&ConfigParamKey::TransactionsRequestLimit(chain)).await?;
        let transactions_result = self.providers.get_transactions_by_address_result(chain, TransactionsRequest::new(payload.value.address, limit)).await?;
        match transactions_result {
            TransactionsResult::Transactions(transactions) => self.producer.publish_transactions(TransactionsPayload::new(chain, transactions)).await,
            TransactionsResult::TransactionRequests(transactions) => self.producer.publish_fetch_transactions(transactions).await,
        }
    }
}
