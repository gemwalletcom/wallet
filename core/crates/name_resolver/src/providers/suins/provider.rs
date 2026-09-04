use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, NameProvider};

use super::client::SuinsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct SuinsProvider {
    client: SuinsClient,
}

impl SuinsProvider {
    pub fn new(url: String) -> Self {
        Self { client: SuinsClient::new(url) }
    }
}

#[async_trait]
impl NameResolver for SuinsProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Suins
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sui"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Sui]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let response = self.client.lookup_name(&query.domain).await?;
        Ok(response.record.and_then(|record| record.target_address))
    }
}
