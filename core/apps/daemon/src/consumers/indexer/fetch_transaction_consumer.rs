use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::TransactionIdRequest;
use settings_chain::ChainProviders;
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

pub struct FetchTransactionConsumer {
    pub providers: ChainProviders,
    pub producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTransactionConsumer {
    pub fn new(providers: ChainProviders, producer: StreamProducer, cacher: CacherClient) -> Self {
        Self { providers, producer, cacher }
    }
}

#[async_trait]
impl MessageConsumer<TransactionIdRequest, usize> for FetchTransactionConsumer {
    async fn should_process(&self, payload: &TransactionIdRequest) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchTransaction(payload.chain.as_ref(), &payload.hash)).await
    }

    async fn process(&self, payload: TransactionIdRequest) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.chain;
        let Some(transaction) = self.providers.get_transaction_by_hash(payload).await? else {
            return Ok(0);
        };
        self.producer.publish_transactions(TransactionsPayload::new(chain, vec![transaction])).await
    }
}
