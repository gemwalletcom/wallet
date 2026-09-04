use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::LensClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct LensProvider {
    client: LensClient,
}

impl LensProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: LensClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for LensProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Lens
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["lens"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Ethereum, Chain::Polygon]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let record = self.client.get_username(&query.name).await?;
        Ok(record.username.and_then(|username| username.linked_to))
    }
}
