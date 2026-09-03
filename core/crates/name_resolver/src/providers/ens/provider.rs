use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::EnsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct EnsProvider {
    client: EnsClient,
}

impl EnsProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: EnsClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for EnsProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Ens
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["eth", "com", "xyz", "dev"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::SmartChain,
            Chain::Polygon,
            Chain::Optimism,
            Chain::Arbitrum,
            Chain::Base,
            Chain::AvalancheC,
            Chain::Fantom,
            Chain::Gnosis,
        ]
    }

    async fn resolve(&self, query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let name = query.ascii_domain()?;
        let resolver = self.client.get_resolver(&name).await?;
        if resolver.is_zero() {
            return Ok(None);
        }
        let address = self.client.get_address(&resolver, &name).await?;
        Ok((!address.is_zero()).then(|| address.to_string()))
    }
}
