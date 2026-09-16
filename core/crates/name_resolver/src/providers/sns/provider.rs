use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, NameProvider};

use super::client::SnsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

const RECORD_BSC: &str = "BSC";

pub struct SnsProvider<C> {
    client: SnsClient<C>,
}

impl<C: Client> SnsProvider<C> {
    pub fn new(client: C) -> Self {
        Self { client: SnsClient::new(client) }
    }
}

#[async_trait]
impl<C: Client> NameResolver for SnsProvider<C> {
    fn provider(&self) -> NameProvider {
        NameProvider::Sns
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["sol", "sns"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Solana, Chain::SmartChain]
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        match chain {
            Chain::Solana => Ok(Some(self.client.get_address(&query.domain).await?)),
            Chain::SmartChain => {
                let (domain, _) = query.domain.rsplit_once('.').ok_or("invalid SNS domain")?;
                Ok(Some(self.client.get_record(domain, RECORD_BSC).await?))
            }
            _ => Err(format!("unsupported chain: {chain}").into()),
        }
    }
}
