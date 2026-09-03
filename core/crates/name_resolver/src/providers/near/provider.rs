use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_jsonrpc::JsonRpcClient;
use gem_near::rpc::NearClient;
use primitives::{Chain, NameProvider};

use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct NearProvider {
    client: NearClient<ReqwestClient>,
}

impl NearProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: NearClient::new(JsonRpcClient::new(client)),
        }
    }
}

#[async_trait]
impl NameResolver for NearProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Near
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["near"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Near]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        self.client.get_account(&query.domain).await?;
        Ok(Some(query.domain.clone()))
    }
}
