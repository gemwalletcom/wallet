use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_jsonrpc::JsonRpcClient;
use gem_near::rpc::NearClient;
use primitives::{Chain, NameProvider};
use serde_json::Value;

use crate::model::NameQuery;
use crate::resolver::NameResolver;

const UNKNOWN_ACCOUNT: &str = "UNKNOWN_ACCOUNT";

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
        match self.client.get_account(&query.domain).await {
            Ok(_) => Ok(Some(query.domain.clone())),
            Err(error) if error.cause.as_ref().and_then(|cause| cause.get("name")).and_then(Value::as_str) == Some(UNKNOWN_ACCOUNT) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}
