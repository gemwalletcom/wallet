use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::DidClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct DidProvider {
    client: DidClient,
}

impl DidProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: DidClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for DidProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Did
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["bit"]
    }

    fn chains(&self) -> Vec<Chain> {
        Chain::all()
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let key = format!("address.{}", chain.as_slip44());
        let records = self.client.get_records(&query.domain).await?;
        Ok(records.into_iter().find(|record| record.key == key).map(|record| record.value))
    }
}
