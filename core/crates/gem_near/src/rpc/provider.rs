use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainAccount, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits, ChainTransaction, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::{Chain, Transaction, TransactionIdRequest};

use super::NearClient;

pub trait NearTransactionProvider: ChainTransactions + ChainTransaction {}

impl<T: ChainTransactions + ChainTransaction> NearTransactionProvider for T {}

pub struct NearProvider<C: Client + Clone> {
    client: NearClient<C>,
    transaction_provider: Box<dyn NearTransactionProvider>,
}

impl<C: Client + Clone> NearProvider<C> {
    pub fn new(client: NearClient<C>, transaction_provider: Box<dyn NearTransactionProvider>) -> Self {
        Self { client, transaction_provider }
    }

    pub fn new_rpc_only(client: NearClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl<C: Client + Clone> Deref for NearProvider<C> {
    type Target = NearClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transactions_by_address(request).await
    }
}

#[async_trait]
impl<C: Client + Clone> ChainBlockTransactions for NearProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        match self.client.get_transactions_by_block(block).await {
            Ok(transactions) => Ok(transactions),
            Err(_) if self.client.is_skipped_block(block).await? => Ok(vec![]),
            Err(error) => Err(error),
        }
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransaction for NearProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transaction_by_hash(request).await
    }
}

impl<C: Client + Clone> ChainProvider for NearProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }
}

impl<C: Client + Clone> ChainStaking for NearProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for NearProvider<C> {}
impl<C: Client + Clone> ChainAccount for NearProvider<C> {}
impl<C: Client + Clone> ChainSimulation for NearProvider<C> {}
impl<C: Client + Clone> ChainTraits for NearProvider<C> {}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use gem_client::testkit::MockClient;
    use gem_jsonrpc::client::JsonRpcClient;
    use num_bigint::BigUint;
    use primitives::{TransactionState, TransactionType, asset_constants::NEAR_USDT_ASSET_ID};
    use serde_json::{Value, json};

    use super::*;

    const UNKNOWN_BLOCK: &str = include_str!("../../testdata/block_unknown.json");
    const FINAL_BLOCK: &str = include_str!("../../testdata/block_final.json");
    const USDT_TRANSACTION: &str = include_str!("../../testdata/rpc_usdt_transaction.json");

    fn provider(requests: Arc<Mutex<Vec<Value>>>, block: Option<Value>) -> NearProvider<MockClient> {
        let rpc = MockClient::new().with_post(move |_, body| {
            let request: Value = serde_json::from_slice(body).unwrap();
            requests.lock().unwrap().push(request.clone());
            let response = match (request["method"].as_str().unwrap(), &block) {
                ("block", _) if request["params"]["finality"] == "final" => serde_json::from_str(FINAL_BLOCK).unwrap(),
                ("block", None) => serde_json::from_str(UNKNOWN_BLOCK).unwrap(),
                ("block", Some(block)) => json!({"jsonrpc": "2.0", "id": request["id"], "result": block}),
                ("chunk", _) if request["params"]["chunk_id"] == "current-with-transaction" => json!({
                    "jsonrpc": "2.0",
                    "id": request["id"],
                    "result": {"transactions": [
                        {
                            "hash": "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E",
                            "signer_id": "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3",
                            "actions": [{"FunctionCall": {"method_name": "ft_transfer", "deposit": "1"}}]
                        },
                        {
                            "hash": "4mGUYn3ccbKuQ6wP2Jp9qCs43GFnBxgnpERVjYxCZYzu",
                            "signer_id": "relayer.near",
                            "actions": [{"FunctionCall": {"method_name": "execute_intents", "deposit": "0"}}]
                        }
                    ]}
                }),
                ("chunk", _) => json!({"jsonrpc": "2.0", "id": request["id"], "result": {"transactions": []}}),
                ("tx", _) => serde_json::from_str(USDT_TRANSACTION).unwrap(),
                (method, _) => panic!("unexpected RPC method: {method}"),
            };
            Ok(serde_json::to_vec(&response).unwrap())
        });
        NearProvider::new_rpc_only(NearClient::new(JsonRpcClient::new(rpc)))
    }

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let block = json!({
            "header": {"hash": "block-hash", "height": 211048907, "timestamp": 1786557652797431689_u64},
            "chunks": [
                {"chunk_hash": "current-with-transaction", "height_included": 211048907, "tx_root": "4UHX5Fx4BVeHNMcBdeJbNtB6Ksgcfu9a3T8nqTHVyGdS"},
                {"chunk_hash": "current-empty", "height_included": 211048907, "tx_root": "11111111111111111111111111111111"},
                {"chunk_hash": "stale", "height_included": 211048906, "tx_root": "4UHX5Fx4BVeHNMcBdeJbNtB6Ksgcfu9a3T8nqTHVyGdS"}
            ]
        });

        let transactions = provider(requests.clone(), Some(block)).get_transactions_by_block(211048907).await.unwrap();

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.hash(), "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E");
        assert_eq!(transaction.block_number.as_deref(), Some("211048907"));
        assert_eq!(transaction.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.state, TransactionState::Confirmed);
        assert_eq!(transaction.from, "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3");
        assert_eq!(transaction.to, "bb90f7cd3f611466d4e8aaee55541d5da6881e01a4155bca49041c1d692b4ff8");
        assert_eq!(transaction.value, BigUint::from(99500026u64));
        assert_eq!(transaction.fee, BigUint::parse_bytes(b"411253844391900000000", 10).unwrap());
        assert_eq!(transaction.created_at.timestamp_micros(), 1786557652797431);

        let params = requests.lock().unwrap().iter().map(|request| request["params"].clone()).collect::<Vec<_>>();
        assert_eq!(
            params,
            vec![
                json!({"block_id": 211048907}),
                json!({"chunk_id": "current-with-transaction"}),
                json!({
                    "tx_hash": "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E",
                    "sender_account_id": "e589457354361489a89039b8be6737cbc2db4d13919b6ccf23889a60f3b0d8f3",
                    "wait_until": "EXECUTED"
                }),
            ]
        );
    }

    #[tokio::test]
    async fn test_get_transactions_by_skipped_block() {
        let provider = provider(Arc::default(), None);

        assert!(provider.get_transactions_by_block(212520205).await.unwrap().is_empty());
        assert!(provider.get_transactions_in_blocks(vec![212520205, 212520210]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_transactions_by_future_block() {
        assert!(provider(Arc::default(), None).get_transactions_by_block(212520211).await.is_err());
    }
}
