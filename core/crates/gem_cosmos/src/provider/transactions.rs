use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use futures::{StreamExt, TryStreamExt, stream};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use super::transactions_mapper::{map_transaction_decode, map_transactions};
use crate::rpc::client::CosmosClient;

#[async_trait]
impl<C: Client> ChainTransactions for CosmosClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let response = self.get_block(block.to_string().as_str()).await?;
        let transaction_ids = response
            .block
            .data
            .txs
            .clone()
            .into_iter()
            .filter(|x| x.len() < 1024)
            .flat_map(|x| map_transaction_decode(&x))
            .collect::<Vec<_>>();

        let receipts = stream::iter(transaction_ids)
            .map(|txid| async move { self.get_transaction(txid.clone()).await })
            .buffer_unordered(5)
            .try_collect()
            .await?;

        Ok(map_transactions(self.chain, receipts))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.chain, vec![self.get_transaction(hash).await?]).into_iter().next())
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transactions = self.get_transactions_by_address_with_limit(&address, limit).await?;
        Ok(TransactionsResult::Transactions(map_transactions(self.chain, transactions)))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_CELESTIA_ADDRESS, TEST_TRANSACTION_ID, create_celestia_test_client, create_cosmos_test_client};
    use chain_traits::ChainTransactions;

    #[tokio::test]
    async fn test_celestia_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let transactions = create_celestia_test_client().get_transactions_by_address_with_limit(TEST_CELESTIA_ADDRESS, 1).await?;

        assert!(transactions.len() <= 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_cosmos_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_cosmos_test_client();
        let transaction = client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await?.unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
