use crate::client::NameClient;
use crate::error::NameError;
use crate::model::NameQuery;
use async_trait::async_trait;
use gem_client::{ClientExt, ReqwestClient, reqwest_client};
use primitives::{Chain, NameProvider};
use serde::Deserialize;
use std::error::Error;

const MAINNET_ANS_ROUTER: &str = "0x867ed1f6bf916171b1de3ee92849b8978b7d1b9e0a8cc982a3d19d535dfd9c0c";

#[derive(Debug, Deserialize)]
struct MoveOption<T> {
    vec: Vec<T>,
}

pub struct AptosClient {
    client: ReqwestClient,
}

impl AptosClient {
    pub fn new(url: String) -> Self {
        Self {
            client: ReqwestClient::new(url, reqwest_client()),
        }
    }

    async fn fetch_name(&self, name: &str) -> Result<Vec<MoveOption<String>>, Box<dyn Error + Send + Sync>> {
        let name = name.strip_suffix(".apt").ok_or_else(|| NameError::new("Invalid Aptos name"))?;
        let (subdomain, domain) = name.rsplit_once('.').map_or((None, name), |(subdomain, domain)| (Some(subdomain), domain));
        let request = serde_json::json!({
            "function": format!("{MAINNET_ANS_ROUTER}::router::get_target_addr"),
            "type_arguments": [],
            "arguments": [domain, { "vec": subdomain.into_iter().collect::<Vec<_>>() }],
        });

        Ok(self.client.post("/v1/view", &request).await?)
    }

    fn map_name(response: Vec<MoveOption<String>>, name: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        response
            .into_iter()
            .next()
            .and_then(|option| option.vec.into_iter().next())
            .ok_or_else(|| NameError::new(format!("Aptos name has no target address: {name}")).into())
    }
}

#[async_trait]
impl NameClient for AptosClient {
    fn provider(&self) -> NameProvider {
        NameProvider::Aptos
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response = self.fetch_name(&query.domain).await?;
        Self::map_name(response, &query.domain)
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["apt"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Aptos]
    }
}
