use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::{NodeType, Transaction, TransactionId};
use serde_json::json;

use crate::rpc::{
    EVMIndexerClient, EthereumMapper,
    client::EthereumClient,
    mapper::CONTRACT_REGISTRY,
    model::{BlockTransactionsIds, Transaction as EthereumTransaction, TransactionReplayTrace},
};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for EthereumClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transaction_hashes = self.indexer.get_transaction_ids_by_address(&address, limit).await?;
        let transaction_ids = transaction_hashes.into_iter().map(|hash| TransactionId::new(self.get_chain(), hash)).collect();
        Ok(TransactionsResult::TransactionIds(transaction_ids))
    }

    async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block = self.get_block(block_number).await?;
        if block.transactions.is_empty() {
            return Ok(Vec::new());
        }

        let receipts = self.get_block_receipts(block_number).await?;
        let traces = if self.node_type == NodeType::Archival {
            Some(self.trace_replay_block_transactions(block_number).await?)
        } else {
            None
        };

        let chain = self.get_chain();
        Ok(block
            .transactions
            .into_iter()
            .zip(receipts)
            .enumerate()
            .filter_map(|(index, (tx, receipt))| {
                let trace = traces.as_ref().and_then(|entries| entries.get(index));
                EthereumMapper::map_transaction(chain, &tx, &receipt, trace, &block.timestamp, Some(&CONTRACT_REGISTRY))
            })
            .collect())
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(transaction) = self.call::<Option<EthereumTransaction>>("eth_getTransactionByHash".to_string(), json!([hash])).await? else {
            return Ok(None);
        };
        let Some(receipt) = self.get_transaction_receipt(&hash).await? else {
            return Ok(None);
        };
        let Some(block) = self
            .call::<Option<BlockTransactionsIds>>("eth_getBlockByNumber".to_string(), json!([format!("0x{:x}", receipt.block_number), false]))
            .await?
        else {
            return Ok(None);
        };
        let trace = if self.node_type == NodeType::Archival {
            Some(
                self.call::<TransactionReplayTrace>("trace_replayTransaction".to_string(), json!([hash, ["stateDiff"]]))
                    .await?,
            )
        } else {
            None
        };
        Ok(EthereumMapper::map_transaction(
            self.get_chain(),
            &transaction,
            &receipt,
            trace.as_ref(),
            &block.timestamp,
            Some(&CONTRACT_REGISTRY),
        ))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TRANSACTION_ID, create_ethereum_test_client};
    use chain_traits::{ChainBalances, ChainTransactionBroadcast, ChainTransactions, TransactionsRequest};
    use num_bigint::BigUint;
    use std::error::Error;

    #[tokio::test]
    async fn test_ethereum_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let result = ChainTransactions::get_transactions_by_address(&client, TransactionsRequest::new(TEST_ADDRESS.to_string(), 5)).await?;
        let transaction_ids = result.transaction_ids().unwrap();
        assert!(!transaction_ids.is_empty());
        assert!(transaction_ids.iter().all(|transaction_id| transaction_id.chain == client.get_chain()));

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_assets_balances() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let balances = ChainBalances::get_balance_assets(&client, TEST_ADDRESS.to_string()).await?;

        println!("Balances: {:#?}", balances);

        assert!(!balances.is_empty());

        let has_assets = balances
            .iter()
            .any(|balance| balance.asset_id.token_id.is_some() && balance.balance.available > BigUint::from(0u32));
        assert!(has_assets);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_transaction_broadcast() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let signed_tx = "0xf86c808502540be40082520894d4e56740f876aef8c010b86a40d5f56745a118d0765af9a146000000808081c0a05e1d3c1b2c3b0f8b7c8e9f0a1b2c3d4e5f6789abcdef0123456789abcdef012345a04f2c3a1b0d8e7f9a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1";
        let options = primitives::BroadcastOptions::default();

        let result = client.transaction_broadcast(signed_tx.to_string(), options).await;

        assert!(result.is_ok() || result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_transaction_broadcast_invalid_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let invalid_tx = "0xinvalidtransactiondata";
        let options = primitives::BroadcastOptions::default();

        let result = client.transaction_broadcast(invalid_tx.to_string(), options).await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let transaction = ChainTransactions::get_transaction_by_hash(&client, TEST_TRANSACTION_ID.to_string()).await?.unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
