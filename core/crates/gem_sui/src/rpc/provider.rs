use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};

use super::SuiClient;

struct EmptyProvider;

#[async_trait]
impl ChainTransactions for EmptyProvider {
    async fn get_transactions_by_address(&self, _request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        Ok(TransactionsResult::Transactions(Vec::new()))
    }
}

pub struct SuiProvider {
    client: SuiClient,
    transactions_by_address_provider: Box<dyn ChainTransactions>,
}

impl SuiProvider {
    pub fn new(client: SuiClient, transactions_by_address_provider: Box<dyn ChainTransactions>) -> Self {
        Self {
            client,
            transactions_by_address_provider,
        }
    }

    pub fn new_rpc_only(client: SuiClient) -> Self {
        Self::new(client, Box::new(EmptyProvider))
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
        self.transactions_by_address_provider.get_transactions_by_address(request).await
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
