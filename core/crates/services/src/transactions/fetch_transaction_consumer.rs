use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use chain_providers::ChainProviders;
use primitives::{Transaction, TransactionId, TransactionIdRequest};
use storage::{Database, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};
use swapper::{SwapResultRequest, swapper::GemSwapper};

use crate::fetch_throttle::{FetchThrottle, ThrottledFetch};
use crate::transactions::transaction_with_swap_result;

pub struct FetchTransactionConsumer {
    pub providers: ChainProviders,
    pub swapper: Arc<GemSwapper>,
    pub producer: StreamProducer,
    pub throttle: Arc<dyn FetchThrottle>,
    pub database: Database,
}

impl FetchTransactionConsumer {
    pub fn new(providers: ChainProviders, swapper: Arc<GemSwapper>, producer: StreamProducer, throttle: Arc<dyn FetchThrottle>, database: Database) -> Self {
        Self {
            providers,
            swapper,
            producer,
            throttle,
            database,
        }
    }

    async fn stored_transaction(&self, id: TransactionId) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        let hash = id.hash.clone();
        let transactions = self.database.run(move |client| client.get_transactions_by_hash(&hash)).await?;
        Ok(transactions.into_iter().find(|transaction| transaction.id == id))
    }
}

#[async_trait]
impl MessageConsumer<TransactionIdRequest, usize> for FetchTransactionConsumer {
    async fn should_consume(&self, payload: &TransactionIdRequest) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle
            .try_start(ThrottledFetch::Transaction {
                chain: payload.chain.as_ref(),
                hash: &payload.hash,
            })
            .await
    }

    async fn consume(&self, payload: TransactionIdRequest) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let (chain, swap_provider) = (payload.chain, payload.swap_provider);
        let id = TransactionId::new(chain, payload.hash.clone());
        let transaction = match self.providers.get_transaction_by_hash(payload).await? {
            Some(transaction) => transaction,
            None => match self.stored_transaction(id).await? {
                Some(transaction) => transaction,
                None => return Ok(0),
            },
        };
        let transaction = match swap_provider {
            Some(provider) => {
                let result = self.swapper.get_swap_result(provider, &SwapResultRequest::from(&transaction)).await?;
                transaction_with_swap_result(transaction, result)
            }
            None => transaction,
        };
        self.producer.publish_transactions(TransactionsPayload::new(chain, vec![transaction])).await
    }
}
