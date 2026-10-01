use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    models::{BlockTransaction, SingleTransaction},
    provider::transaction_mapper::{map_block_transactions, map_transaction},
    rpc::{SolanaIndexer, SolanaProvider, constants::MISSING_BLOCKS_ERRORS},
};

#[async_trait]
impl<C: Client + Clone> ChainBlockTransactions for SolanaProvider<C> {
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
}

#[async_trait]
impl<C: Client + Clone> ChainTransaction for SolanaProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let transaction: Option<SingleTransaction> = self.get_transaction(&request.hash).await?;
        match transaction {
            Some(transaction) => Ok(map_single_transaction(transaction)),
            None => self.indexer.get_transaction_by_hash(request).await,
        }
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransaction for SolanaIndexer<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(self.get_transaction(&request.hash).await?.and_then(map_single_transaction))
    }
}

fn map_single_transaction(transaction: SingleTransaction) -> Option<Transaction> {
    let block_transaction = BlockTransaction {
        meta: transaction.meta,
        transaction: transaction.transaction,
    };
    map_transaction(&block_transaction, transaction.block_time)
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for SolanaIndexer<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transaction_ids = self.get_transaction_ids_by_address(&address, limit).await?;
        Ok(TransactionsResult::TransactionRequests(
            transaction_ids.into_iter().map(|transaction_id| TransactionIdRequest::new(primitives::Chain::Solana, transaction_id, None)).collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{Chain, testkit::json::load_json_rpc_result};
    use serde_json::Value;

    use super::*;
    use crate::{method::GET_TRANSACTION, rpc::SolanaClient};

    const MAYAN_SWIFT_DEPOSIT: &str = "2VhEGwgr8foH7m3Xo4yciuMYvPDvcYjvGRyj4mNbQfjkKoLj7DmgA9FfWWi2HzhSW1mLHNNKExXpNcUnC8TgLcFA";

    #[tokio::test]
    async fn test_get_transaction_by_hash_missing_on_node_uses_indexer() {
        let node = mock_jsonrpc_client(|method, _| {
            assert_eq!(method, GET_TRANSACTION);
            Ok(Value::Null)
        });
        let indexer = mock_jsonrpc_client(|method, _| {
            assert_eq!(method, GET_TRANSACTION);
            Ok(load_json_rpc_result(include_str!("../../testdata/mayan_swift_deposit_token.json")))
        });
        let provider = SolanaProvider::new(SolanaClient::new(node), Box::new(SolanaIndexer::new(indexer)));

        let transaction = provider.get_transaction_by_hash(TransactionIdRequest::new(Chain::Solana, MAYAN_SWIFT_DEPOSIT.to_string(), None)).await.unwrap().unwrap();

        assert_eq!(transaction.hash(), MAYAN_SWIFT_DEPOSIT);
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
        let result = client.get_transactions_by_address(TransactionsRequest::new(TEST_SOLANA_SENDER.to_string(), 100)).await.unwrap();
        let transactions = result.transaction_requests().unwrap();

        println!("Address: {}, transactions count: {}", TEST_SOLANA_SENDER, transactions.len());
        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_solana_get_transaction_by_hash() {
        let client = create_solana_test_client();
        let transaction = client
            .get_transaction_by_hash(TransactionIdRequest::new(primitives::Chain::Solana, TEST_TRANSACTION_ID.to_string(), None))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
    }
}
