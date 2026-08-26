use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainBlockTransactions, ChainTransaction, TransactionIdRequest};
use gem_client::Client;
use primitives::Transaction;
use serde_json::{Value, from_value};

use crate::jsonrpc::EthereumRpc;
use crate::rpc::{
    EthereumMapper, EthereumProvider,
    model::{BlockHeader, Transaction as RpcTransaction, TransactionReceipt},
};

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainBlockTransactions for EthereumProvider<C> {
    async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(self
            .get_transactions_by_block_with_receipts(block_number)
            .await?
            .into_iter()
            .map(|(transaction, _)| transaction)
            .collect())
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumProvider<C> {
    pub async fn get_transactions_by_block_with_receipts(&self, block_number: u64) -> Result<Vec<(Transaction, TransactionReceipt)>, Box<dyn Error + Sync + Send>> {
        let Some(block) = self.get_block(block_number).await? else {
            return Err(format!("block {block_number} not available").into());
        };
        if block.transactions.is_empty() {
            return Ok(Vec::new());
        }

        let receipts = self.get_block_receipts(block_number).await?;
        Ok(block
            .transactions
            .into_iter()
            .zip(receipts)
            .filter_map(|(transaction, receipt)| {
                EthereumMapper::map_transaction_with_parser(self.get_chain(), &transaction, &receipt, &block.timestamp, self.provider.protocol_parser())
                    .map(|transaction| (transaction, receipt))
            })
            .collect())
    }

    pub async fn get_transaction_with_receipt(&self, request: TransactionIdRequest) -> Result<Option<(Transaction, TransactionReceipt)>, Box<dyn Error + Sync + Send>> {
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
        Ok(
            EthereumMapper::map_transaction_with_parser(self.get_chain(), &transaction, &receipt, &timestamp, self.provider.protocol_parser())
                .map(|transaction| (transaction, receipt)),
        )
    }
}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransaction for EthereumProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(self.get_transaction_with_receipt(request).await?.map(|(transaction, _)| transaction))
    }
}

#[cfg(all(test, feature = "rpc"))]
mod tests {
    use chain_traits::{ChainBlockTransactions, ChainTransaction, TransactionIdRequest};
    use gem_client::{ClientError, testkit::MockClient};
    use gem_jsonrpc::{JsonRpcClient, testkit::mock_jsonrpc_client};
    use primitives::{Chain, EVMChain, testkit::json::load_json_rpc_result};
    use serde_json::{Value, json};

    use crate::{
        method,
        rpc::{EthereumClient, EthereumProvider},
    };

    #[tokio::test]
    async fn test_get_transactions_by_block_null() {
        let client = mock_jsonrpc_client(|_, _| Ok(Value::Null));
        let provider = EthereumProvider::new_rpc_only(EthereumClient::new(client, EVMChain::Ink));

        let error = provider.get_transactions_by_block(54181824).await.unwrap_err();

        assert_eq!(error.to_string(), "block 54181824 not available");
    }

    #[tokio::test]
    async fn test_get_transaction_by_hash_batches_known_block() {
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
        let client = EthereumProvider::new_rpc_only(EthereumClient::new(JsonRpcClient::new(transport), EVMChain::Arbitrum));

        let transaction = client
            .get_transaction_by_hash(TransactionIdRequest::new(
                Chain::Arbitrum,
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
    use crate::provider::testkit::{
        TEST_ADDRESS, TEST_TRANSACTION_ID, create_ethereum_test_asset_balance_provider, create_ethereum_test_client, create_ethereum_test_transactions_by_address_provider,
    };
    use chain_traits::{ChainTransaction, ChainTransactionBroadcast, ChainTransactions, TransactionIdRequest, TransactionsRequest};
    use num_bigint::BigUint;
    use primitives::{BroadcastOptions, Chain};
    use std::error::Error;

    use crate::rpc::AssetBalanceProvider;

    #[tokio::test]
    async fn test_ethereum_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let transactions_by_address = create_ethereum_test_transactions_by_address_provider();
        let result = ChainTransactions::get_transactions_by_address(&transactions_by_address, TransactionsRequest::new(TEST_ADDRESS.to_string(), 5)).await?;
        let transaction_requests = result.transaction_requests().unwrap();
        assert!(!transaction_requests.is_empty());
        assert!(transaction_requests.iter().all(|transaction| transaction.chain == Chain::Ethereum));

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_assets_balances() -> Result<(), Box<dyn Error + Send + Sync>> {
        let asset_balances = create_ethereum_test_asset_balance_provider();
        let balances = AssetBalanceProvider::get_asset_balances(&asset_balances, TEST_ADDRESS.to_string()).await?;

        println!("Balances: {:#?}", balances);

        assert!(!balances.is_empty());

        let has_assets = balances
            .iter()
            .any(|balance| balance.asset_id.token_id.is_some() && balance.balance.available > BigUint::from(0u32));
        assert!(has_assets);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_transaction_broadcast_invalid_data() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let invalid_tx = "0xinvalidtransactiondata";
        let options = BroadcastOptions::default();

        let result = client.transaction_broadcast(invalid_tx.to_string(), options).await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_transaction_by_hash() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let transaction = ChainTransaction::get_transaction_by_hash(&client, TransactionIdRequest::new(client.get_chain(), TEST_TRANSACTION_ID.to_string(), None))
            .await?
            .unwrap();

        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        Ok(())
    }
}
