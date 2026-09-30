use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainTransaction, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};

use super::SuiClient;

pub trait SuiTransactionsIndexer: ChainTransactions + ChainTransaction {}

impl<T: ChainTransactions + ChainTransaction> SuiTransactionsIndexer for T {}

pub struct SuiProvider {
    client: SuiClient,
    pub(crate) indexer: Box<dyn SuiTransactionsIndexer>,
}

impl SuiProvider {
    pub fn new(client: SuiClient, indexer: Box<dyn SuiTransactionsIndexer>) -> Self {
        Self { client, indexer }
    }

    pub fn new_rpc_only(client: SuiClient) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl Deref for SuiProvider {
    type Target = SuiClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl ChainTransactions for SuiProvider {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.indexer.get_transactions_by_address(request).await
    }
}

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_only_provider_returns_empty_transactions() {
        let provider = SuiProvider::new_rpc_only(SuiClient::new("https://example.com"));
        let transactions = match provider.get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), 10)).await.unwrap() {
            TransactionsResult::Transactions(transactions) => transactions,
            TransactionsResult::TransactionRequests(_) => panic!("RPC-only provider must return an empty transaction list"),
        };

        assert!(transactions.is_empty());
    }
}
