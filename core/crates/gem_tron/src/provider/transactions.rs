use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use super::transactions_mapper::{map_transaction, map_transactions_by_block};
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for TronClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block_transactions(block).await?;
        if block_data.transactions.is_empty() {
            return Ok(vec![]);
        }

        let receipts = self.get_block_transactions_receipts(block).await?;
        Ok(map_transactions_by_block(self.get_chain(), block_data, receipts))
    }

    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let hash = request.hash;
        let Some(receipt) = self.get_transaction_receipt(hash.clone()).await? else {
            return Ok(None);
        };
        Ok(map_transaction(self.get_chain(), self.get_transaction(hash).await?, receipt))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transactions = self.trongrid_client.get_transactions_by_address(&address, limit).await?.data;

        Ok(TransactionsResult::TransactionRequests(
            transactions
                .into_iter()
                .map(|transaction| TransactionIdRequest::new(self.get_chain(), transaction.transaction_id, None))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANSACTIONS_RESPONSE: &str = include_str!("../../testdata/transactions_by_address.json");
    const ADDRESS: &str = "TBKwjUtXVsX1r724C1V52nocBgtioDjx9u";
    const LAGGING_TRANSACTION_ID: &str = "e5c5dc535b267134024e887c00b1522426661b1b5ae6efb76606f4d83bca1a81";
    const INCOMING_TRANSACTION_ID: &str = "7b633cd06802d7117a7202650c7580516c742ce1e20d43ba736ab8da1a02cd8f";

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let client = TronClient::mock(|_| Ok(TRANSACTIONS_RESPONSE.as_bytes().to_vec()));
        let result = client.get_transactions_by_address(TransactionsRequest::new(ADDRESS.to_string(), 4)).await.unwrap();
        let transactions = result.transaction_requests().unwrap();
        assert_eq!(transactions.len(), 4);
        assert!(transactions.iter().any(|transaction| transaction.hash == LAGGING_TRANSACTION_ID));
        assert!(transactions.iter().any(|transaction| transaction.hash == INCOMING_TRANSACTION_ID));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TRANSACTION_ID, create_test_client};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let tron_client = create_test_client();

        let latest_block = tron_client.get_block_latest_number().await.unwrap();
        let block_number = latest_block - 25;
        let transactions = tron_client.get_transactions_by_block(block_number).await.unwrap();

        assert!(latest_block > 0);
        assert!(!transactions.is_empty());

        if let Some(transaction) = transactions.first() {
            assert!(!transaction.id.hash.is_empty());
        }
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let tron_client = create_test_client();
        let result = tron_client
            .get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string(), 2))
            .await
            .unwrap();
        let transactions = result.transaction_requests().unwrap();
        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let tron_client = create_test_client();
        let transaction = tron_client
            .get_transaction_by_hash(TransactionIdRequest::new(primitives::Chain::Tron, TEST_TRANSACTION_ID.to_string(), None))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
    }
}
