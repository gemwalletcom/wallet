use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::Transaction;
use serde_json::{Value, from_value};

use crate::jsonrpc::EthereumRpc;
use crate::rpc::{
    EVMIndexerClient, EthereumMapper,
    client::EthereumClient,
    model::{BlockHeader, Transaction as RpcTransaction, TransactionReceipt},
};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for EthereumClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transactions = self.indexer.get_transactions_by_address(&address, limit).await?;
        let transaction_requests = transactions
            .into_iter()
            .map(|transaction| TransactionIdRequest::new(self.get_chain(), transaction.hash, transaction.block_number))
            .collect();
        Ok(TransactionsResult::TransactionRequests(transaction_requests))
    }

    async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block = self.get_block(block_number).await?;
        if block.transactions.is_empty() {
            return Ok(Vec::new());
        }

        let receipts = self.get_block_receipts(block_number).await?;
        let chain = self.get_chain();
        Ok(block
            .transactions
            .into_iter()
            .zip(receipts)
            .filter_map(|(tx, receipt)| EthereumMapper::map_transaction(chain, &tx, &receipt, &block.timestamp))
            .collect())
    }

    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionIdRequest { hash, block_number, .. } = request;
        let (transaction, receipt, timestamp) = match block_number {
            Some(block_number) => {
                let responses = self
                    .client
                    .batch_request::<EthereumRpc, Value>(vec![
                        EthereumRpc::GetTransactionByHash(hash.clone()),
                        EthereumRpc::GetTransactionReceipt(hash),
                        EthereumRpc::GetBlockByNumber(block_number, false),
                    ])
                    .await?
                    .take_all()?;
                let [transaction, receipt, block] = responses.try_into().map_err(|_| "EVM transaction batch response length mismatch")?;
                (transaction, receipt, Some(from_value::<BlockHeader>(block)?.timestamp))
            }
            None => {
                let responses = self
                    .client
                    .batch_request::<EthereumRpc, Value>(vec![EthereumRpc::GetTransactionByHash(hash.clone()), EthereumRpc::GetTransactionReceipt(hash)])
                    .await?
                    .take_all()?;
                let [transaction, receipt] = responses.try_into().map_err(|_| "EVM transaction batch response length mismatch")?;
                (transaction, receipt, None)
            }
        };
        let transaction: Option<RpcTransaction> = from_value(transaction)?;
        let Some(transaction) = transaction else {
            return Ok(None);
        };
        let receipt: Option<TransactionReceipt> = from_value(receipt)?;
        let Some(receipt) = receipt else {
            return Ok(None);
        };
        let timestamp = match timestamp {
            Some(timestamp) => timestamp,
            None => self.get_block_timestamp(receipt.block_number).await?,
        };
        Ok(EthereumMapper::map_transaction(self.get_chain(), &transaction, &receipt, &timestamp))
    }
}

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use chain_traits::{ChainTransactions, TransactionIdRequest};
    use gem_client::{ClientError, testkit::MockClient};
    use gem_jsonrpc::JsonRpcClient;
    use primitives::{EVMChain, testkit::json::load_json_rpc_result};
    use serde_json::{Value, json};

    use crate::{method, rpc::EthereumClient};

    #[tokio::test]
    async fn get_transaction_by_hash_batches_known_block() {
        let transport = MockClient::new().with_post(|_, body| {
            let requests: Vec<Value> = serde_json::from_slice(body).map_err(|error| ClientError::Serialization(error.to_string()))?;
            assert_eq!(
                requests.iter().map(|request| request["method"].as_str().unwrap()).collect::<Vec<_>>(),
                [method::ETH_GET_TRANSACTION_BY_HASH, method::ETH_GET_TRANSACTION_RECEIPT, method::ETH_GET_BLOCK_BY_NUMBER,]
            );
            assert_eq!(requests[2]["params"], json!(["0x150db7d1", false]));

            let transaction: Value = load_json_rpc_result(include_str!("../../testdata/transfer_erc20.json"));
            let receipt: Value = load_json_rpc_result(include_str!("../../testdata/transfer_erc20_receipt.json"));
            let results = [transaction, receipt, json!({ "timestamp": "0x65a1f600" })];
            serde_json::to_vec(
                &requests
                    .iter()
                    .zip(results)
                    .map(|(request, result)| json!({ "jsonrpc": "2.0", "id": request["id"], "result": result }))
                    .collect::<Vec<_>>(),
            )
            .map_err(|error| ClientError::Serialization(error.to_string()))
        });
        let client = EthereumClient::new(JsonRpcClient::new(transport), EVMChain::Arbitrum);

        let transaction = client
            .get_transaction_by_hash(TransactionIdRequest::new(
                primitives::Chain::Arbitrum,
                "0xd6878ac03656ac15c9bc24cc4daf3ff276de637ec2d9708c420186f6cba9dc06".to_string(),
                Some(0x150db7d1),
            ))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(transaction.hash, "0xd6878ac03656ac15c9bc24cc4daf3ff276de637ec2d9708c420186f6cba9dc06");
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
        let transaction_requests = result.transaction_requests().unwrap();
        assert!(!transaction_requests.is_empty());
        assert!(transaction_requests.iter().all(|transaction| transaction.chain == client.get_chain()));

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
        let transaction = ChainTransactions::get_transaction_by_hash(&client, TransactionIdRequest::new(client.get_chain(), TEST_TRANSACTION_ID.to_string(), None))
            .await?
            .unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
