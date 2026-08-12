use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_near::rpc::NearClient;
use primitives::{Chain, NameProvider};

use crate::client::NameClient;
use crate::model::NameQuery;

pub struct NearNameClient {
    client: NearClient<ReqwestClient>,
}

impl NearNameClient {
    pub fn new(url: String) -> Self {
        Self {
            client: NearClient::new(JsonRpcClient::new_reqwest(url)),
        }
    }
}

#[async_trait]
impl NameClient for NearNameClient {
    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.client.get_account(&query.domain).await?;
        Ok(query.domain.clone())
    }

    fn provider(&self) -> NameProvider {
        NameProvider::Near
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["near"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Near]
    }
}
