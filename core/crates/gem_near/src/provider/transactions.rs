use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};

use gem_client::Client;

use crate::rpc::NearIndexer;

#[async_trait]
impl<C: Client> ChainTransactions for NearIndexer<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest {
            address, limit, from_timestamp, ..
        } = request;
        let transactions = self.get_transactions_by_address(&address, limit, from_timestamp).await?;
        Ok(TransactionsResult::Transactions(transactions))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use std::error::Error;

    use chain_traits::{ChainTransactions, TransactionsRequest};

    use crate::provider::testkit::{TEST_HISTORY_ADDRESS, create_near_test_client};

    #[tokio::test]
    async fn test_near_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let result = create_near_test_client()
            .get_transactions_by_address(TransactionsRequest::new(TEST_HISTORY_ADDRESS.to_string(), 3))
            .await?;
        let transactions = result.transactions().ok_or("expected full NEAR transactions")?;

        assert_eq!(transactions.len(), 3);
        Ok(())
    }
}
