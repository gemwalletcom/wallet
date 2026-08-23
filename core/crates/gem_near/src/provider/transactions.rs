use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest, TransactionsResult};

use gem_client::Client;
use primitives::{Transaction, TransactionIdRequest};

use crate::rpc::NearIndexer;

#[async_trait]
impl<C: Client> ChainBlockTransactions for NearIndexer<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        NearIndexer::get_transactions_by_block(self, block).await
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for NearIndexer<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        NearIndexer::get_transaction_by_hash(self, request).await
    }
}

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

    use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest};
    use primitives::{Chain, TransactionIdRequest, asset_constants::NEAR_USDT_ASSET_ID};

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

    #[tokio::test]
    async fn test_near_get_transaction_by_hash() -> Result<(), Box<dyn Error + Send + Sync>> {
        let hash = "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E";
        let transaction = create_near_test_client()
            .get_transaction_by_hash(TransactionIdRequest::new(Chain::Near, hash.to_string(), Some(211048907)))
            .await?
            .ok_or("expected NEAR transaction")?;

        assert_eq!(transaction.hash, hash);
        assert_eq!(transaction.block_number.as_deref(), Some("211048907"));
        assert_eq!(transaction.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(transaction.value, "99500026");
        Ok(())
    }

    #[tokio::test]
    async fn test_near_get_transactions_by_block() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_near_test_client();
        let block_number = 212506690;
        let transactions = client.get_transactions_by_block(block_number).await?;
        let expected_block_number = block_number.to_string();

        assert_eq!(transactions.len(), 7);
        assert!(
            transactions
                .iter()
                .all(|transaction| transaction.block_number.as_deref() == Some(expected_block_number.as_str()))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_near_get_transactions_by_missing_block() -> Result<(), Box<dyn Error + Send + Sync>> {
        let transactions = create_near_test_client().get_transactions_by_block(999999999).await?;

        assert!(transactions.is_empty());
        Ok(())
    }
}
