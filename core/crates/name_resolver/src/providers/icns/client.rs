use std::error::Error;

use gem_client::ReqwestClient;
use gem_cosmos::rpc::CosmosClient;
use primitives::chain_cosmos::CosmosChain;
use serde_json::json;

use super::model::Record;

const RESOLVER_ADDRESS: &str = "osmo1xk0s8xgktn9x5vwcgtjdxqzadg88fgn33p8u9cnpdxwemvxscvast52cdd";

pub struct IcnsClient {
    client: CosmosClient<ReqwestClient>,
}

impl IcnsClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: CosmosClient::new(CosmosChain::Osmosis, client),
        }
    }

    pub async fn get_record(&self, domain: &str) -> Result<Record, Box<dyn Error + Send + Sync>> {
        let query = json!({ "address_by_icns": { "icns": domain } });
        self.client.get_contract_smart_query(RESOLVER_ADDRESS, &query).await
    }
}
