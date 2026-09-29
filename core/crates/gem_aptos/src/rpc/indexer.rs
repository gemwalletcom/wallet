use std::error::Error;

use gem_client::{Client, ClientExt, Target};
use primitives::graphql::GraphqlData;
use serde::Serialize;
use serde_json::{Value, json};

use crate::models::{FungibleAssetActivities, FungibleAssetActivity};

const DEPOSITS_QUERY: &str = "query($owner: String!, $after: bigint!, $limit: Int!) { fungible_asset_activities(where: { owner_address: { _eq: $owner }, type: { _like: \"%Deposit%\" }, transaction_version: { _gt: $after } }, order_by: { transaction_version: asc }, limit: $limit) { transaction_version asset_type amount } }";

#[derive(Debug, Serialize)]
struct GraphqlRequest {
    query: &'static str,
    variables: Value,
}

#[derive(Debug)]
struct AptosIndexerTarget;

impl Target for AptosIndexerTarget {
    fn path(&self) -> String {
        "/v1/graphql".to_string()
    }
}

#[derive(Clone, Debug)]
pub struct AptosIndexer<C: Client> {
    client: C,
}

impl<C: Client> AptosIndexer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_deposits(&self, owner: &str, after_version: u64, limit: usize) -> Result<Vec<FungibleAssetActivity>, Box<dyn Error + Send + Sync>> {
        let request = GraphqlRequest {
            query: DEPOSITS_QUERY,
            variables: json!({ "owner": owner, "after": after_version, "limit": limit }),
        };
        let response: GraphqlData<FungibleAssetActivities> = self.client.post(AptosIndexerTarget, &request).await?;
        if let Some(error) = response.errors.and_then(|errors| errors.into_iter().next()) {
            return Err(error.message.into());
        }
        Ok(response.data.ok_or("missing Aptos indexer data")?.fungible_asset_activities)
    }
}
