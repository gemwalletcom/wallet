use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::BasenamesClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct BasenamesProvider {
    client: BasenamesClient,
}

impl BasenamesProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: BasenamesClient::new(client),
        }
    }
}

#[async_trait]
impl NameResolver for BasenamesProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Basenames
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["base.eth"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Base]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let address = self.client.get_address(&query.domain).await?;
        Ok((!address.is_zero()).then(|| address.to_string()))
    }
}
