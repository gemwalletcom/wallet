#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use primitives::Transaction;

use crate::provider::transactions_mapper::{map_transaction, map_transaction_blocks};
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl ChainTransactions for SuiClient {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        let transaction_blocks = self.get_checkpoint_transactions(block, None).await?;
        Ok(map_transaction_blocks(transaction_blocks))
    }

    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        let hash = request.hash;
        Ok(map_transaction(self.get_transaction(hash).await?))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn std::error::Error + Sync + Send>> {
        let transactions = self.indexer.get_transactions_by_address(&request.address, request.limit).await?;
        Ok(TransactionsResult::Transactions(transactions.into_iter().filter_map(map_transaction).collect()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainState, ChainTransactionState, ChainTransactions, TransactionIdRequest, TransactionsRequest};
    use primitives::{TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = ChainTransactions::get_transactions_by_block(&client, latest_block - 1).await?;

        println!("Transactions in block {}: {}", latest_block - 1, transactions.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let result = client.get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string(), 100)).await?;
        let transactions = result.transactions().ok_or("Expected Sui transactions")?;

        assert!(!transactions.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(latest_block - 1).await?;
        let transaction_id = transactions.transactions.first().ok_or("No Sui transaction found in latest checkpoint")?;
        let request = TransactionStateRequest::mock_with_id(transaction_id);
        let status = client.get_transaction_status(request).await?;

        println!("Transaction status: {:?}", status);

        assert!(status.state == TransactionState::Confirmed);
        assert!(status.changes.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let mut transaction = None;
        let mut transaction_id = None;
        let mut transaction_block = None;

        for block in (latest_block.saturating_sub(20)..latest_block).rev() {
            let transactions = client.get_transactions_by_block(block).await?;
            for digest in transactions.transactions {
                if let Some(mapped) = ChainTransactions::get_transaction_by_hash(&client, TransactionIdRequest::new(primitives::Chain::Sui, digest.to_string(), None)).await? {
                    transaction = Some(mapped);
                    transaction_id = Some(digest);
                    transaction_block = Some(block);
                    break;
                }
            }
            if transaction.is_some() {
                break;
            }
        }
        let transaction = transaction.ok_or("No mappable Sui transaction found in recent checkpoints")?;
        let transaction_id = transaction_id.ok_or("No Sui transaction hash found")?;
        let transaction_block = transaction_block.ok_or("No Sui transaction block found")?;

        println!("Mappable Sui transaction in block {}: {}", transaction_block, transaction_id);

        assert_eq!(transaction.hash, transaction_id);
        Ok(())
    }
}
