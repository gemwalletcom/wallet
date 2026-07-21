use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use serde::Deserialize;
use serde_json::json;

use crate::COMMITMENT_CONFIRMED;

#[derive(Debug, Deserialize)]
struct Transactions {
    data: Vec<Transaction>,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    signature: String,
}

#[derive(Debug, Clone)]
pub struct SolanaIndexer<C: Client + Clone> {
    client: JsonRpcClient<C>,
}

impl<C: Client + Clone> SolanaIndexer<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client }
    }

    pub(crate) async fn get_transaction_ids_by_address(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let transactions: Transactions = self
            .client
            .call(
                "getTransactionsForAddress",
                json!([address, {
                    "transactionDetails": "signatures",
                    "sortOrder": "desc",
                    "limit": limit,
                    "commitment": COMMITMENT_CONFIRMED
                }]),
            )
            .await?;
        Ok(transactions.data.into_iter().map(|signature| signature.signature).collect())
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;

    use super::*;

    #[tokio::test]
    async fn test_get_transaction_ids_by_address() {
        let client = mock_jsonrpc_client(|method, params| {
            assert_eq!(method, "getTransactionsForAddress");
            assert_eq!(
                params,
                &json!(["address", {
                    "transactionDetails": "signatures",
                    "sortOrder": "desc",
                    "limit": 2,
                    "commitment": COMMITMENT_CONFIRMED
                }])
            );
            Ok(load_json(include_str!("../../testdata/alchemy_get_transactions_for_address.json")))
        });
        let indexer = SolanaIndexer::new(client);

        let transaction_ids = indexer.get_transaction_ids_by_address("address", 2).await.unwrap();

        assert_eq!(transaction_ids, vec!["signature-1", "signature-2"]);
    }
}
