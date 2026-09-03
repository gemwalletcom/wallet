use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::EthsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct EthsProvider {
    client: EthsClient,
}

impl EthsProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: EthsClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for EthsProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Tree
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["tree", "eths", "honk"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Ethereum, Chain::Polygon, Chain::SmartChain]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        Ok(Some(self.client.get_record(&query.domain).await?.owner))
    }
}
