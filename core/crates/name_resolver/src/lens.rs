use async_trait::async_trait;
use gem_client::reqwest_client;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::client::NameClient;
use crate::model::NameQuery;
use primitives::{Chain, NameProvider};

#[derive(Debug, Deserialize, Serialize)]
struct Data<T> {
    data: T,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    pub username: Option<Username>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Username {
    pub linked_to: Option<String>,
}

const LENS_NAMESPACE: &str = "0x1aA55B9042f08f45825dC4b651B64c9F98Af4615";

pub struct LensClient {
    api_url: String,
    client: Client,
}

impl LensClient {
    pub fn new(api_url: String) -> Self {
        let client = reqwest_client();
        Self { api_url, client }
    }
}

#[async_trait]
impl NameClient for LensClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Lens
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let query = format!(
            "query {{ username(request: {{ username: {{ localName: \"{}\", namespace: \"{LENS_NAMESPACE}\" }} }}) {{ linkedTo }} }}",
            query.name
        );
        let query = serde_json::json!({
            "query": query,
        });

        let address = self
            .client
            .post(&self.api_url)
            .json(&query)
            .send()
            .await?
            .json::<Data<Record>>()
            .await?
            .data
            .username
            .and_then(|username| username.linked_to);

        address.ok_or("address not found".into())
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["lens"]
    }

    fn chains(&self) -> Vec<Chain> {
        // Add all evm chains?
        vec![Chain::Ethereum, Chain::Polygon]
    }
}
