use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    provider::transactions_mapper::{map_transaction_by_hash, map_transactions},
    rpc::AlgorandIndexer,
};

#[async_trait]
impl<C: Client> ChainBlockTransactions for AlgorandIndexer<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block = self.get_block(block).await?;
        Ok(map_transactions(block.transactions))
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for AlgorandIndexer<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(self.get_transaction(&request.hash).await?.and_then(map_transaction_by_hash))
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandIndexer<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, .. } = request;
        let transactions = self.get_account_transactions(&address).await?;
        Ok(TransactionsResult::Transactions(map_transactions(transactions.transactions)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::{ClientError, testkit::MockClient};
    use primitives::Chain;

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let indexer = AlgorandIndexer::new(MockClient::new().with_get(|_| {
            Err(ClientError::Http {
                status: 404,
                body: br#"{"message":"no transaction found for transaction id"}"#.to_vec(),
            })
        }));

        let transaction = indexer
            .get_transaction_by_hash(TransactionIdRequest::new(Chain::Algorand, "MN2OHOKE4PFHRO35HQBJIBKNUJ3HJVHQTB3HVJ43ATNPD4ULW67A".to_string(), None))
            .await
            .unwrap();

        assert_eq!(transaction, None);
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::{ChainBlockTransactions, ChainState, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest};

    #[tokio::test]
    async fn test_algorand_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(latest_block - 1).await?;
        println!("Transactions in block {}: {}", latest_block - 1, transactions.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let result = client.get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string(), 100)).await?;
        let transactions = result.transactions().unwrap();
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());

        assert!(!transactions.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let transaction = client.get_transaction_by_hash(TransactionIdRequest::new(primitives::Chain::Algorand, TEST_TRANSACTION_ID.to_string(), None)).await?.unwrap();

        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
        Ok(())
    }
}
