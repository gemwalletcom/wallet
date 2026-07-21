#[cfg(feature = "reqwest")]
use std::sync::Arc;
use std::{error::Error, fmt::Debug};

use async_trait::async_trait;
#[cfg(feature = "reqwest")]
use gem_client::ReqwestClient;
use gem_client::{Client, ClientExt};
use primitives::graphql::GraphqlData;
use serde::Deserialize;

use super::indexer_mapper::{GraphqlTransaction, map_transaction};
use crate::models::Digest;

pub const SUI_GRAPHQL_URL: &str = "https://graphql.mainnet.sui.io/graphql";

const TRANSACTIONS_BY_ADDRESS_QUERY: &str = "query GetTransactionsByAddress($address: SuiAddress!, $limit: Int!, $before: String) { transactions(last: $limit, before: $before, filter: { affectedAddress: $address }) { nodes { digest effects { status timestamp gasEffects { gasObject { owner { ... on AddressOwner { address { address } } } } gasSummary { computationCost storageCost storageRebate nonRefundableStorageFee } } balanceChanges(first: 50) { nodes { owner { address } coinType { repr } amount } } events(first: 50) { nodes { contents { type { repr } json } transactionModule { package { address } } } } } } pageInfo { hasPreviousPage startCursor } } }";
const TRANSACTIONS_PAGE_SIZE: usize = 50;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionsData {
    transactions: TransactionConnection,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransactionConnection {
    nodes: Vec<GraphqlTransaction>,
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
    has_previous_page: bool,
    start_cursor: Option<String>,
}

#[async_trait]
pub(crate) trait SuiIndexerClient: Send + Sync + Debug {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<Digest>, Box<dyn Error + Send + Sync>>;
}

#[derive(Clone, Debug)]
pub struct SuiIndexer<C: Client> {
    client: C,
}

impl<C: Client> SuiIndexer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> SuiIndexerClient for SuiIndexer<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<Digest>, Box<dyn Error + Send + Sync>> {
        let mut transactions = Vec::with_capacity(limit);
        let mut before = None;

        while transactions.len() < limit {
            let page_size = (limit - transactions.len()).min(TRANSACTIONS_PAGE_SIZE);
            let request = serde_json::json!({
                "operationName": "GetTransactionsByAddress",
                "variables": {
                    "address": address,
                    "limit": page_size,
                    "before": before,
                },
                "query": TRANSACTIONS_BY_ADDRESS_QUERY,
            });
            let response: GraphqlData<TransactionsData> = self.client.post("", &request).await?;
            if let Some(error) = response.errors.and_then(|errors| errors.into_iter().next()) {
                return Err(error.message.into());
            }
            let page = response.data.ok_or("missing Sui GraphQL transaction data")?.transactions;
            transactions.extend(page.nodes.into_iter().rev().map(map_transaction).collect::<Result<Vec<_>, _>>()?);
            if !page.page_info.has_previous_page {
                break;
            }
            before = Some(page.page_info.start_cursor.ok_or("missing Sui GraphQL transaction cursor")?);
        }

        Ok(transactions)
    }
}

#[cfg(feature = "reqwest")]
pub(super) fn default_indexer() -> Arc<dyn SuiIndexerClient> {
    let client = ReqwestClient::new(SUI_GRAPHQL_URL.to_string(), gem_client::reqwest_client());
    Arc::new(SuiIndexer::new(client))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        sync::{Arc, Mutex},
    };

    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_get_transactions_by_address() {
        let responses = Arc::new(Mutex::new(VecDeque::from([
            include_str!("../../testdata/transactions_by_address_page_1.json").as_bytes().to_vec(),
            include_str!("../../testdata/transactions_by_address_page_2.json").as_bytes().to_vec(),
        ])));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let responses_for_client = responses.clone();
        let requests_for_client = requests.clone();
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "");
            requests_for_client.lock().unwrap().push(serde_json::from_slice::<serde_json::Value>(body).unwrap());
            Ok(responses_for_client.lock().unwrap().pop_front().unwrap())
        });

        let transactions = SuiIndexer::new(client).get_transactions_by_address("address", 51).await.unwrap();
        let requests = requests.lock().unwrap();

        assert_eq!(
            transactions.iter().map(|transaction| transaction.digest.as_str()).collect::<Vec<_>>(),
            vec!["newest", "older", "oldest"]
        );
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0]["operationName"], "GetTransactionsByAddress");
        assert_eq!(requests[0]["query"], TRANSACTIONS_BY_ADDRESS_QUERY);
        assert_eq!(requests[0]["variables"]["address"], "address");
        assert_eq!(requests[0]["variables"]["limit"], 50);
        assert_eq!(requests[0]["variables"]["before"], serde_json::Value::Null);
        assert_eq!(requests[1]["variables"]["address"], "address");
        assert_eq!(requests[1]["variables"]["limit"], 49);
        assert_eq!(requests[1]["variables"]["before"], "cursor");
        assert!(responses.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_transactions_by_address_error() {
        let client = MockClient::new().with_post(|_, _| Ok(include_str!("../../testdata/transactions_by_address_error.json").as_bytes().to_vec()));

        let error = SuiIndexer::new(client).get_transactions_by_address("invalid", 1).await.unwrap_err();

        assert_eq!(error.to_string(), "invalid address");
    }
}
