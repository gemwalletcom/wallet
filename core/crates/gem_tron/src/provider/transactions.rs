use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use super::transactions_mapper::{map_transaction, map_transactions_by_address, map_transactions_by_block};
use crate::rpc::client::TronClient;

#[async_trait]
impl<C: Client + Clone> ChainTransactions for TronClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_data = self.get_block_tranactions(block).await?;
        if block_data.transactions.is_empty() {
            return Ok(vec![]);
        }

        let receipts = self.get_block_tranactions_reciepts(block).await?;
        Ok(map_transactions_by_block(self.get_chain(), block_data, receipts))
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(receipt) = self.get_transaction_reciept(hash.clone()).await? else {
            return Ok(None);
        };
        Ok(map_transaction(self.get_chain(), self.get_transaction(hash).await?, receipt))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let limit = limit.unwrap_or(20);
        let transactions = self.trongrid_client.get_transactions_by_address(&address, limit).await?.data;

        if transactions.is_empty() {
            return Ok(vec![]);
        }

        let futures = transactions.iter().map(|transaction| self.get_transaction_reciept(transaction.transaction_id.clone()));
        let receipts = futures::future::try_join_all(futures).await?;
        let (transactions, receipts) = transactions
            .into_iter()
            .zip(receipts)
            .filter_map(|(transaction, receipt)| receipt.map(|receipt| (transaction, receipt)))
            .unzip();

        Ok(map_transactions_by_address(transactions, receipts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::mock_tron_client;
    use gem_client::ClientError;

    const TRANSACTIONS_RESPONSE: &str = include_str!("../../testdata/transactions_by_address.json");
    const RECEIPTS_RESPONSE: &str = include_str!("../../testdata/transactions_by_address_receipts.json");
    const ADDRESS: &str = "TBKwjUtXVsX1r724C1V52nocBgtioDjx9u";
    const LAGGING_TRANSACTION_ID: &str = "e5c5dc535b267134024e887c00b1522426661b1b5ae6efb76606f4d83bca1a81";
    const INCOMING_TRANSACTION_ID: &str = "7b633cd06802d7117a7202650c7580516c742ce1e20d43ba736ab8da1a02cd8f";

    async fn mock_fetch(receipt_handler: impl Fn(&str) -> Result<Vec<u8>, ClientError> + Send + Sync + 'static) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let client = mock_tron_client(move |path| {
            if path.contains("/transactions?") {
                Ok(TRANSACTIONS_RESPONSE.as_bytes().to_vec())
            } else {
                receipt_handler(path)
            }
        });
        client.get_transactions_by_address(TransactionsRequest::new(ADDRESS.to_string()).with_limit(4)).await
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let receipts: Vec<serde_json::Value> = serde_json::from_str(RECEIPTS_RESPONSE).unwrap();

        let valid_receipts = receipts.clone();
        let transactions = mock_fetch(move |path| {
            if path.contains(LAGGING_TRANSACTION_ID) {
                Ok(b"{}".to_vec())
            } else {
                let receipt = valid_receipts.iter().find(|receipt| path.contains(receipt["id"].as_str().unwrap())).unwrap();
                Ok(serde_json::to_vec(receipt).unwrap())
            }
        })
        .await
        .unwrap();
        assert_eq!(transactions.len(), 3);
        assert!(!transactions.iter().any(|transaction| transaction.hash == LAGGING_TRANSACTION_ID));
        assert!(transactions.iter().any(|transaction| transaction.hash == INCOMING_TRANSACTION_ID));

        let outage = mock_fetch(|path| {
            if path.contains(LAGGING_TRANSACTION_ID) {
                Err(ClientError::Http { status: 503, body: vec![] })
            } else {
                Ok(b"{}".to_vec())
            }
        })
        .await;
        assert!(outage.is_err());

        let malformed = mock_fetch(|path| {
            if path.contains(LAGGING_TRANSACTION_ID) {
                Ok(br#"{"unexpected":"shape"}"#.to_vec())
            } else {
                Ok(b"{}".to_vec())
            }
        })
        .await;
        assert!(malformed.is_err());
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
        let transactions = tron_client
            .get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string()).with_limit(2))
            .await
            .unwrap();

        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash() {
        let tron_client = create_test_client();
        let transaction = tron_client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await.unwrap().unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
    }
}
