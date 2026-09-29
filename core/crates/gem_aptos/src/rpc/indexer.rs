use std::error::Error;

use gem_client::{Client, ClientExt, Target};
use primitives::graphql::GraphqlData;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::models::{FungibleAssetActivities, FungibleAssetActivity, UserTransaction, UserTransactions};

const DEPOSITS_QUERY: &str = "query($owner: String!, $after: bigint!, $limit: Int!) { fungible_asset_activities(where: { owner_address: { _eq: $owner }, type: { _like: \"%Deposit%\" }, transaction_version: { _gt: $after } }, order_by: { transaction_version: asc }, limit: $limit) { transaction_version owner_address asset_type amount type is_transaction_success } }";
const USER_TRANSACTIONS_QUERY: &str = "query($versions: [bigint!]!) { user_transactions(where: { version: { _in: $versions } }) { version sender entry_function_id_str } }";
const ACTIVITIES_QUERY: &str = "query($versions: [bigint!]!, $owners: [String!]!) { fungible_asset_activities(where: { transaction_version: { _in: $versions }, owner_address: { _in: $owners }, is_gas_fee: { _eq: false } }, order_by: { transaction_version: asc }) { transaction_version owner_address asset_type amount type is_transaction_success } }";

#[derive(Debug, Serialize)]
struct GraphqlRequest {
    query: &'static str,
    variables: Value,
}

#[derive(Debug)]
struct AptosIndexerTarget;

impl Target for AptosIndexerTarget {
    fn path(&self) -> String {
        String::new()
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

    async fn query<T: DeserializeOwned + Send>(&self, query: &'static str, variables: Value) -> Result<T, Box<dyn Error + Send + Sync>> {
        let response: GraphqlData<T> = self.client.post(AptosIndexerTarget, &GraphqlRequest { query, variables }).await?;
        if let Some(error) = response.errors.and_then(|errors| errors.into_iter().next()) {
            return Err(error.message.into());
        }
        Ok(response.data.ok_or("missing Aptos indexer data")?)
    }

    pub async fn get_deposits(&self, owner: &str, after_version: u64, limit: usize) -> Result<Vec<FungibleAssetActivity>, Box<dyn Error + Send + Sync>> {
        let data: FungibleAssetActivities = self.query(DEPOSITS_QUERY, json!({ "owner": owner, "after": after_version, "limit": limit })).await?;
        Ok(data.fungible_asset_activities)
    }

    pub async fn get_user_transactions(&self, versions: &[u64]) -> Result<Vec<UserTransaction>, Box<dyn Error + Send + Sync>> {
        let data: UserTransactions = self.query(USER_TRANSACTIONS_QUERY, json!({ "versions": versions })).await?;
        Ok(data.user_transactions)
    }

    pub async fn get_activities(&self, versions: &[u64], owners: &[String]) -> Result<Vec<FungibleAssetActivity>, Box<dyn Error + Send + Sync>> {
        let data: FungibleAssetActivities = self.query(ACTIVITIES_QUERY, json!({ "versions": versions, "owners": owners })).await?;
        Ok(data.fungible_asset_activities)
    }
}
