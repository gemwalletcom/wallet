use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::SnsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

const RECORD_BSC: &str = "BSC";

pub struct SnsProvider {
    client: SnsClient,
}

impl SnsProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: SnsClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for SnsProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Sns
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sol"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Solana, Chain::SmartChain]
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::Solana => Ok(Some(self.client.get_address(&query.domain).await?)),
            Chain::SmartChain => Ok(Some(self.client.get_record(&query.domain, RECORD_BSC).await?)),
            _ => Err(format!("unsupported chain: {chain}").into()),
        }
    }
}
