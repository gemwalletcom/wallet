use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::Transaction;

use super::ZcashProvider;

#[async_trait]
impl<C: Client> ChainBlockTransactions for ZcashProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        self.client.get_transactions_by_block(block).await
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for ZcashProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        self.client.get_transaction_by_hash(request).await
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for ZcashProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.client.get_transactions_by_address(request).await
    }
}
