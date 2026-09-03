use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::InjectiveClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct InjectiveProvider {
    client: InjectiveClient,
}

impl InjectiveProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: InjectiveClient::new(client),
        }
    }
}

#[async_trait]
impl NameResolver for InjectiveProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Injective
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["inj"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Injective]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        Ok(Some(self.client.get_record(&query.domain).await?.address))
    }
}
