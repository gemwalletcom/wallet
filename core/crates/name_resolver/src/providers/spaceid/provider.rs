use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::SpaceIdClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

const CODE_OK: i32 = 0;

pub struct SpaceIdProvider {
    client: SpaceIdClient,
}

impl SpaceIdProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: SpaceIdClient::new(client),
        }
    }
}

#[async_trait]
impl NameResolver for SpaceIdProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Spaceid
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["bnb", "arb"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::SmartChain, Chain::Arbitrum]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let record = self.client.get_address(&query.suffix, &query.domain).await?;
        if record.code != CODE_OK {
            return Err(format!("Space ID request failed with code: {}", record.code).into());
        }
        Ok(Some(record.address))
    }
}
