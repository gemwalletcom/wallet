use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use gem_jsonrpc::JsonRpcClient;
use gem_near::rpc::NearClient;
use primitives::{Chain, NameProvider};

use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct NearProvider<C: Client + Clone> {
    client: NearClient<C>,
}

impl<C: Client + Clone> NearProvider<C> {
    pub fn new(client: C) -> Self {
        Self {
            client: NearClient::new(JsonRpcClient::new(client)),
        }
    }
}

#[async_trait]
impl<C: Client + Clone> NameResolver for NearProvider<C> {
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
            Err(error) if error.payload.is_unknown_account() => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use gem_client::{ClientError, testkit::MockClient};
    use serde_json::Value;

    use super::*;

    #[tokio::test]
    async fn test_missing_account_requires_unknown_account_cause() {
        for (cause, expected) in [(Some("UNKNOWN_ACCOUNT"), Ok(None::<&str>)), (Some("UNKNOWN_BLOCK"), Err("Server error (-32000)")), (None, Err("Server error (-32000)"))] {
            for status in [200, 400, 500] {
                let mut response: Value = serde_json::from_str(include_str!("../../../../gem_near/testdata/account_unknown.json")).unwrap();
                response["error"]["cause"] = cause.map(|name| serde_json::json!({"name": name})).unwrap_or(Value::Null);
                let body = serde_json::to_vec(&response).unwrap();
                let provider = NearProvider::new(MockClient::new().with_post(move |_, _| if status == 200 { Ok(body.clone()) } else { Err(ClientError::Http { status, body: body.clone() }) }));

                assert_eq!(
                    provider.resolve(&NameQuery::new("missing.near"), Chain::Near).await.map_err(|error| error.to_string()),
                    expected.map(|address| address.map(str::to_string)).map_err(str::to_string)
                );
            }
        }
    }
}
