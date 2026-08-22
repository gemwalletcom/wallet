use crate::{
    constants::{EMPTY_TRANSACTION_ROOT, RPC_CONCURRENCY, UNKNOWN_BLOCK_ERROR},
    jsonrpc::NearRpc,
    models::{Account, AccountAccessKey, Block, BroadcastResult, Chunk, GasPrice, NodeStatus, ProtocolConfig},
    rpc::mapper::map_transaction,
};
use futures::{StreamExt, TryStreamExt, stream};
use gem_client::Client;
use gem_encoding::encode_base64;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcError};
use primitives::{Chain, Transaction};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::error::Error;

#[derive(Deserialize)]
struct ContractCallResult {
    result: Vec<u8>,
}

#[derive(Debug)]
pub struct NearClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
    pub chain: Chain,
}

impl<C: Client + Clone> NearClient<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Near }
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, JsonRpcError> {
        self.client.request(NearRpc::GetAccount(address.to_string())).await
    }

    pub async fn call_function<T: Serialize, R: DeserializeOwned>(&self, contract_id: &str, method_name: &str, args: &T) -> Result<R, Box<dyn Error + Sync + Send>> {
        let args = serde_json::to_vec(args)?;
        let response: ContractCallResult = self
            .client
            .request(NearRpc::CallFunction {
                contract_id: contract_id.to_string(),
                method_name: method_name.to_string(),
                args_base64: encode_base64(&args),
            })
            .await?;
        Ok(serde_json::from_slice(&response.result)?)
    }

    pub async fn get_account_access_key(&self, address: &str, public_key: &str) -> Result<AccountAccessKey, JsonRpcError> {
        self.client
            .request(NearRpc::GetAccountAccessKey {
                address: address.to_string(),
                public_key: public_key.to_string(),
            })
            .await
    }

    pub async fn get_latest_block(&self) -> Result<Block, JsonRpcError> {
        self.client.request(NearRpc::GetLatestBlock).await
    }

    pub async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block: Block = match self.client.request(NearRpc::GetBlock(block_number)).await {
            Ok(block) => block,
            Err(error) if error.cause.as_ref().and_then(|cause| cause.get("name")).and_then(|name| name.as_str()) == Some(UNKNOWN_BLOCK_ERROR) => return Ok(vec![]),
            Err(error) => return Err(Box::new(error)),
        };
        let chunk_requests = block
            .chunks
            .into_iter()
            .filter(|chunk| chunk.height_included == block_number && chunk.tx_root != EMPTY_TRANSACTION_ROOT)
            .map(|chunk| NearRpc::GetChunk(chunk.chunk_hash))
            .collect();
        let chunks: Vec<Chunk> = self.request_all(chunk_requests).await?;
        let transaction_requests = chunks
            .into_iter()
            .flat_map(|chunk| chunk.transactions)
            .map(|transaction| NearRpc::GetTransactionStatus {
                transaction_hash: transaction.hash,
                sender_account_id: transaction.signer_id,
            })
            .collect();
        let transactions: Vec<BroadcastResult> = self.request_all(transaction_requests).await?;

        transactions
            .into_iter()
            .map(|transaction| {
                let state = transaction.state();
                let fee = transaction.fee();
                let receipts = transaction.receipts_outcome.into_iter().map(|receipt| receipt.outcome).collect();
                map_transaction(transaction.transaction, receipts, block_number, block.header.timestamp, state, fee)
            })
            .collect()
    }

    async fn request_all<R: DeserializeOwned + Send>(&self, requests: Vec<NearRpc>) -> Result<Vec<R>, JsonRpcError> {
        stream::iter(requests)
            .map(|request| self.client.request(request))
            .buffered(RPC_CONCURRENCY)
            .try_collect()
            .await
    }

    pub async fn get_gas_price(&self) -> Result<GasPrice, JsonRpcError> {
        self.client.request(NearRpc::GetGasPrice).await
    }

    pub async fn get_protocol_config(&self) -> Result<ProtocolConfig, JsonRpcError> {
        self.client.request(NearRpc::GetProtocolConfig).await
    }

    pub async fn get_status(&self) -> Result<NodeStatus, JsonRpcError> {
        self.client.request(NearRpc::GetStatus).await
    }

    pub async fn broadcast_transaction(&self, signed_transaction: &str) -> Result<BroadcastResult, JsonRpcError> {
        self.client.request(NearRpc::SendTransaction(signed_transaction.to_string())).await
    }

    pub async fn get_transaction(&self, transaction_hash: &str, sender_account_id: &str) -> Result<BroadcastResult, JsonRpcError> {
        self.client
            .request(NearRpc::GetTransactionStatus {
                transaction_hash: transaction_hash.to_string(),
                sender_account_id: sender_account_id.to_string(),
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gem_client::testkit::MockClient;
    use primitives::{TransactionType, asset_constants::NEAR_USDT_ASSET_ID};
    use serde_json::{Value, json};

    use super::*;

    #[tokio::test]
    async fn gets_block_transactions_from_rpc() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let recorded_requests = requests.clone();
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "");
            let request: Value = serde_json::from_slice(body).unwrap();
            recorded_requests.lock().unwrap().push(request.clone());
            let result = match request["method"].as_str().unwrap() {
                "block" => json!({
                    "header": {
                        "hash": "block-hash",
                        "height": 211048907,
                        "timestamp": 1786557652797431689_u64
                    },
                    "chunks": [
                        {"chunk_hash": "current-with-transaction", "height_included": 211048907, "tx_root": "transactions"},
                        {"chunk_hash": "current-empty", "height_included": 211048907, "tx_root": "11111111111111111111111111111111"},
                        {"chunk_hash": "stale", "height_included": 211048906, "tx_root": "transactions"}
                    ]
                }),
                "chunk" if request["params"]["chunk_id"] == "current-with-transaction" => json!({
                    "transactions": [
                        {
                            "hash": "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E",
                            "signer_id": "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3"
                        },
                        {
                            "hash": "HmhQCa1cL9ab7bQNnv1fJxz25fD6WZ5L4UPgrrD9k4FK",
                            "signer_id": "abound.near"
                        }
                    ]
                }),
                "chunk" => json!({"transactions": []}),
                "tx" if request["params"]["tx_hash"] == "HmhQCa1cL9ab7bQNnv1fJxz25fD6WZ5L4UPgrrD9k4FK" => {
                    serde_json::from_str(include_str!("../../testdata/rpc_create_account_transaction.json")).unwrap()
                }
                "tx" => serde_json::from_str(include_str!("../../testdata/rpc_usdt_transaction.json")).unwrap(),
                method => panic!("unexpected RPC method: {method}"),
            };
            Ok(serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": request["id"], "result": result})).unwrap())
        });
        let near = NearClient::new(JsonRpcClient::new(client));

        let transactions = near.get_transactions_by_block(211048907).await.unwrap();

        assert_eq!(transactions.len(), 2);
        let transaction = transactions
            .iter()
            .find(|transaction| transaction.hash == "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E")
            .unwrap();
        assert_eq!(transaction.hash, "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E");
        assert_eq!(transaction.block_number.as_deref(), Some("211048907"));
        assert_eq!(transaction.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.value, "99500026");
        assert_eq!(transaction.fee, "411253844391900000000");

        let create_account = transactions
            .iter()
            .find(|transaction| transaction.hash == "HmhQCa1cL9ab7bQNnv1fJxz25fD6WZ5L4UPgrrD9k4FK")
            .unwrap();
        assert_eq!(create_account.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(create_account.from, "abound.near");
        assert_eq!(create_account.to, "6968519963136087096.abound.near");
        assert_eq!(create_account.value, "0");
        assert_eq!(create_account.fee, "7091964925000000000000");

        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 4);
        assert_eq!(requests[0]["params"], json!({"block_id": 211048907}));
        assert_eq!(requests[1]["params"], json!({"chunk_id": "current-with-transaction"}));
        assert_eq!(
            requests[2]["params"],
            json!({
                "tx_hash": "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E",
                "sender_account_id": "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3",
                "wait_until": "EXECUTED"
            })
        );
        assert_eq!(
            requests[3]["params"],
            json!({
                "tx_hash": "HmhQCa1cL9ab7bQNnv1fJxz25fD6WZ5L4UPgrrD9k4FK",
                "sender_account_id": "abound.near",
                "wait_until": "EXECUTED"
            })
        );
    }

    #[tokio::test]
    async fn skips_unknown_block() {
        let client = MockClient::new().with_post(|_, _| Ok(include_bytes!("../../testdata/rpc_unknown_block.json").to_vec()));
        let near = NearClient::new(JsonRpcClient::new(client));

        assert!(near.get_transactions_by_block(212357397).await.unwrap().is_empty());
    }
}
