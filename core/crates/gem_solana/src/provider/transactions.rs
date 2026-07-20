use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;
use primitives::{Transaction, TransactionId};

use crate::{
    models::{BlockTransaction, SingleTransaction},
    provider::transaction_mapper::{map_block_transactions, map_transaction},
    rpc::{client::SolanaClient, constants::MISSING_BLOCKS_ERRORS},
};

#[async_trait]
impl<C: Client + Clone> ChainTransactions for SolanaClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        match self.get_block_transactions(block).await {
            Ok(block_transactions) => Ok(map_block_transactions(&block_transactions)),
            Err(error) => {
                if MISSING_BLOCKS_ERRORS.contains(&error.code) {
                    return Ok(vec![]);
                }
                Err(Box::new(error))
            }
        }
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(transaction) = self
            .rpc_call::<Option<SingleTransaction>>("getTransaction", serde_json::json!([hash, { "encoding": "json", "maxSupportedTransactionVersion": 0 }]))
            .await?
        else {
            return Ok(None);
        };
        let block_transaction = BlockTransaction {
            meta: transaction.meta,
            transaction: transaction.transaction,
        };
        Ok(map_transaction(&block_transaction, transaction.block_time))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let signatures = self.get_signatures_for_address(&address, limit).await?;
        Ok(TransactionsResult::TransactionIds(
            signatures.into_iter().map(|signature| TransactionId::new(self.get_chain(), signature.signature)).collect(),
        ))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_TRANSACTION_ID, create_solana_test_client};
    use chain_traits::ChainState;
    use primitives::testkit::signer_mock::TEST_SOLANA_SENDER;

    #[tokio::test]
    async fn test_solana_get_transactions_by_block() {
        let client = create_solana_test_client();

        let latest_block = client.get_block_latest_number().await.unwrap();
        let transactions = client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_solana_get_transactions_by_address() {
        let client = create_solana_test_client();
        let result = client
            .get_transactions_by_address(TransactionsRequest::new(TEST_SOLANA_SENDER.to_string(), 100))
            .await
            .unwrap();
        let transaction_ids = result.transaction_ids().unwrap();

        println!("Address: {}, transactions count: {}", TEST_SOLANA_SENDER, transaction_ids.len());
        assert!(!transaction_ids.is_empty());
    }

    #[tokio::test]
    async fn test_solana_get_transaction_by_hash() {
        let client = create_solana_test_client();
        let transaction = client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await.unwrap().unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
    }
}
