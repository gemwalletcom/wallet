use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest, TransactionsResult};
use primitives::{Chain, Transaction};
use std::error::Error;

use crate::{
    provider::transactions_mapper,
    rpc::{PolkadotIndexer, PolkadotProvider},
};
use gem_client::Client;

#[async_trait]
impl<C: Client> ChainBlockTransactions for PolkadotProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block(block as i64).await?;
        Ok(transactions_mapper::map_transactions(Chain::Polkadot, block_data))
    }
}

impl<C: Client> ChainTransaction for PolkadotProvider<C> {}

#[async_trait]
impl<C: Client> ChainTransactions for PolkadotIndexer<C> {
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

    use crate::provider::testkit::{TEST_ADDRESS, create_polkadot_test_client};
    use chain_traits::{ChainTransactionState, ChainTransactions, TransactionsRequest};
    use primitives::{TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_polkadot_get_transaction_status_failed() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let request = TransactionStateRequest::mock_with_id("0x3a9dda661cbdfe12e15c623cd14abf3da64d4bcbe11c0c776def748713c2248b").with_block_number(27_830_222);

        let result = client.get_transaction_status(request).await?;

        assert_eq!(result.state, TransactionState::Failed);
        assert!(result.changes.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_polkadot_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let result = create_polkadot_test_client()
            .get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string(), 3))
            .await?;
        let transactions = result.transactions().ok_or("expected full Polkadot transactions")?;

        assert_eq!(transactions.len(), 3);
        Ok(())
    }
}
