use std::error::Error;

use alloy_ens::namehash;
use gem_client::ReqwestClient;
use gem_cosmos::rpc::CosmosClient;
use primitives::chain_cosmos::CosmosChain;
use serde_json::json;

use super::model::ResolverRecord;

const RESOLVER_ADDRESS: &str = "inj1x9m0hceug9qylcyrrtwqtytslv2jrph433thgu";

pub struct InjectiveClient {
    client: CosmosClient<ReqwestClient>,
}

impl InjectiveClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: CosmosClient::new(CosmosChain::Injective, client),
        }
    }

    pub async fn get_record(&self, domain: &str) -> Result<ResolverRecord, Box<dyn Error + Send + Sync>> {
        let query = json!({ "address": { "node": namehash(domain).to_vec() } });
        self.client.get_contract_smart_query(RESOLVER_ADDRESS, &query).await
    }
}
