use std::error::Error;

use async_trait::async_trait;
use gem_aptos::{AptosClient, MoveOption, ViewRequest};
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};
use serde_json::json;

use crate::model::NameQuery;
use crate::resolver::NameResolver;

const ANS_ROUTER_ADDRESS: &str = "0x867ed1f6bf916171b1de3ee92849b8978b7d1b9e0a8cc982a3d19d535dfd9c0c";

pub struct AptosProvider {
    client: AptosClient<ReqwestClient>,
}

impl AptosProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: AptosClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for AptosProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Aptos
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["apt"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Aptos]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let name = query.domain.strip_suffix(".apt").ok_or("invalid Aptos name")?;
        let (subdomain, domain) = name.rsplit_once('.').map_or((None, name), |(subdomain, domain)| (Some(subdomain), domain));
        let request = ViewRequest::new(
            format!("{ANS_ROUTER_ADDRESS}::router::get_target_addr"),
            vec![
                json!(domain),
                json!(MoveOption {
                    vec: subdomain.into_iter().collect()
                }),
            ],
        );
        let response: Vec<MoveOption<String>> = self.client.view(request).await?;
        Ok(response.into_iter().next().and_then(|option| option.vec.into_iter().next()))
    }
}
