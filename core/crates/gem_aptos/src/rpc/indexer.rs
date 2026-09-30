use std::error::Error;

use gem_client::{Client, ClientExt, Target};
use primitives::graphql::GraphqlData;
use serde::Serialize;

use crate::models::IndexerTransaction;

const TRANSACTION_QUERY: &str = "query GetTransactionByVersion($version: bigint!) { user_transactions(where: { version: { _eq: $version } }) { sender entry_function_id_str timestamp } fungible_asset_activities(where: { transaction_version: { _eq: $version } }) { owner_address asset_type amount type is_gas_fee is_transaction_success } }";

#[derive(Clone, Debug)]
struct AptosIndexerTarget;

impl Target for AptosIndexerTarget {
    fn path(&self) -> String {
        String::new()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphqlRequest {
    operation_name: &'static str,
    query: &'static str,
    variables: TransactionVariables,
}

#[derive(Serialize)]
struct TransactionVariables {
    version: u64,
}

#[derive(Clone, Debug)]
pub struct AptosIndexer<C: Client> {
    client: C,
}

impl<C: Client> AptosIndexer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_transaction_by_version(&self, version: u64) -> Result<Option<IndexerTransaction>, Box<dyn Error + Send + Sync>> {
        let body = GraphqlRequest {
            operation_name: "GetTransactionByVersion",
            query: TRANSACTION_QUERY,
            variables: TransactionVariables { version },
        };
        let response: GraphqlData<IndexerTransaction> = self.client.post(AptosIndexerTarget, &body).await?;
        if let Some(error) = response.errors.and_then(|errors| errors.into_iter().next()) {
            return Err(error.message.into());
        }
        Ok(response.data.filter(|transaction| !transaction.user_transactions.is_empty()))
    }
}
