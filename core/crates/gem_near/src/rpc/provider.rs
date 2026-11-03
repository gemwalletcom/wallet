use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainAccount, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits, ChainTransaction, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::{Chain, Transaction, TransactionIdRequest};

use super::NearClient;

pub trait NearTransactionProvider: ChainTransactions + ChainBlockTransactions + ChainTransaction {}

impl<T: ChainTransactions + ChainBlockTransactions + ChainTransaction> NearTransactionProvider for T {}

pub struct NearProvider<C: Client + Clone> {
    client: NearClient<C>,
    transaction_provider: Box<dyn NearTransactionProvider>,
}

impl<C: Client + Clone> NearProvider<C> {
    pub fn new(client: NearClient<C>, transaction_provider: Box<dyn NearTransactionProvider>) -> Self {
        Self { client, transaction_provider }
    }

    pub fn new_rpc_only(client: NearClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl<C: Client + Clone> Deref for NearProvider<C> {
    type Target = NearClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transactions_by_address(request).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBlockTransactions for NearProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        match self.transaction_provider.get_transactions_by_block(block).await {
            Ok(transactions) => Ok(transactions),
            Err(_) if self.client.is_skipped_block(block).await? => Ok(vec![]),
            Err(error) => Err(error),
        }
    }

    async fn get_transactions_in_blocks(&self, blocks: Vec<u64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let mut transactions = Vec::new();
        for block in blocks {
            transactions.extend(self.get_transactions_by_block(block).await?);
        }
        Ok(transactions)
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransaction for NearProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transaction_by_hash(request).await
    }
}

impl<C: Client + Clone> ChainProvider for NearProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }
}

impl<C: Client + Clone> ChainStaking for NearProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for NearProvider<C> {}
impl<C: Client + Clone> ChainAccount for NearProvider<C> {}
impl<C: Client + Clone> ChainSimulation for NearProvider<C> {}
impl<C: Client + Clone> ChainTraits for NearProvider<C> {}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::client::JsonRpcClient;

    use super::*;
    use crate::rpc::NearIndexer;

    const UNKNOWN_BLOCK: &str = r#"{"jsonrpc":"2.0","id":1,"error":{"name":"HANDLER_ERROR","cause":{"info":{},"name":"UNKNOWN_BLOCK"},"code":-32000,"message":"Server error"}}"#;
    const FINAL_BLOCK: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"header":{"hash":"hash","height":212520210}}}"#;

    fn provider(block: &'static str) -> NearProvider<MockClient> {
        let rpc = MockClient::new().with_post(move |_, body| {
            let is_final_request = String::from_utf8_lossy(body).contains("finality");
            Ok(if is_final_request { FINAL_BLOCK } else { block }.as_bytes().to_vec())
        });
        let indexer = MockClient::new().with_post(|_, _| Ok(br#"{"block":null}"#.to_vec()));
        NearProvider::new(NearClient::new(JsonRpcClient::new(rpc)), Box::new(NearIndexer::new(indexer.clone(), indexer)))
    }

    #[tokio::test]
    async fn test_get_transactions_by_unindexed_block() {
        let error = provider(FINAL_BLOCK).get_transactions_by_block(212520205).await.unwrap_err();

        assert_eq!(error.to_string(), "FastNear block 212520205 is not indexed");
    }

    #[tokio::test]
    async fn test_get_transactions_by_skipped_block() {
        let provider = provider(UNKNOWN_BLOCK);

        assert!(provider.get_transactions_by_block(212520205).await.unwrap().is_empty());
        assert!(provider.get_transactions_in_blocks(vec![212520205, 212520210]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_transactions_by_future_block() {
        assert_eq!(provider(UNKNOWN_BLOCK).get_transactions_by_block(212520211).await.unwrap_err().to_string(), "FastNear block 212520211 is not indexed");
    }
}
