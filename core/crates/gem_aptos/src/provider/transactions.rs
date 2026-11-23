use async_trait::async_trait;
use chain_traits::{BlockTransactionsResult, ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult, get_transactions_by_each_block};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    provider::transactions_mapper::{map_transaction, map_transactions},
    rpc::client::AptosClient,
};

#[async_trait]
impl<C: Client> ChainBlockTransactions for AptosClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_block_transactions(block).await?.transactions))
    }

    async fn get_transactions_in_blocks(&self, blocks: Vec<u64>) -> Vec<BlockTransactionsResult> {
        if blocks.len() < 2 || !blocks.windows(2).all(|pair| pair[1] == pair[0] + 1) {
            return get_transactions_by_each_block(self, blocks).await;
        }
        match self.get_block_range_transactions(blocks[0], blocks[blocks.len() - 1]).await {
            Ok(transactions) => blocks.into_iter().zip(transactions).map(|(block, transactions)| (block, Ok(map_transactions(transactions)))).collect(),
            Err(error) => {
                let message = error.to_string();
                blocks.into_iter().map(|block| (block, Err(message.clone().into()))).collect()
            }
        }
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for AptosClient<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let hash = request.hash;
        Ok(map_transaction(self.get_transaction_by_hash(&hash).await?))
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for AptosClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, .. } = request;
        Ok(TransactionsResult::Transactions(map_transactions(self.get_transactions_by_address(address).await?)))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TRANSACTION_ID, create_aptos_test_client};
    use chain_traits::{ChainBlockTransactions, ChainState, ChainTransaction, TransactionIdRequest};

    #[tokio::test]
    async fn test_aptos_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let _latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(100000).await?;
        println!("Transactions in block 100000: {}", transactions.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_transactions_in_blocks_matches_single_blocks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let (first, last) = (latest_block - 120, latest_block - 101);

        let range = client.get_block_range_transactions(first, last).await?;
        let mut single = Vec::new();
        for block in first..=last {
            single.push(client.get_block_transactions(block).await?.transactions);
        }

        let hashes = |blocks: &[Vec<crate::models::Transaction>]| blocks.iter().map(|block| block.iter().map(|transaction| transaction.hash.clone()).collect::<Vec<_>>()).collect::<Vec<_>>();
        assert_eq!(hashes(&range), hashes(&single));
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string()).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let transaction = ChainTransaction::get_transaction_by_hash(&client, TransactionIdRequest::new(primitives::Chain::Aptos, TEST_TRANSACTION_ID.to_string(), None))
            .await?
            .unwrap();

        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
        Ok(())
    }
}
