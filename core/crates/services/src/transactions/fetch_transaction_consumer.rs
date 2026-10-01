use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use chain_providers::ChainProviders;
use primitives::TransactionIdRequest;
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};
use swapper::swapper::GemSwapper;

use crate::transactions::transaction_with_swap_result;

pub struct FetchTransactionConsumer {
    pub providers: ChainProviders,
    pub swapper: Arc<GemSwapper>,
    pub producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTransactionConsumer {
    pub fn new(providers: ChainProviders, swapper: Arc<GemSwapper>, producer: StreamProducer, cacher: CacherClient) -> Self {
        Self { providers, swapper, producer, cacher }
    }
}

#[async_trait]
impl MessageConsumer<TransactionIdRequest, usize> for FetchTransactionConsumer {
    async fn should_consume(&self, payload: &TransactionIdRequest) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchTransaction(payload.chain.as_ref(), &payload.hash)).await
    }

    async fn consume(&self, payload: TransactionIdRequest) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let (chain, swap_provider) = (payload.chain, payload.swap_provider);
        let Some(transaction) = self.providers.get_transaction_by_hash(payload).await? else {
            return Ok(0);
        };
        let transaction = match swap_provider {
            Some(provider) => {
                let result = self.swapper.get_swap_result(chain, provider, &transaction.id.hash).await?;
                transaction_with_swap_result(transaction, result)
            }
            None => transaction,
        };
        self.producer.publish_transactions(TransactionsPayload::new(chain, vec![transaction])).await
    }
}
