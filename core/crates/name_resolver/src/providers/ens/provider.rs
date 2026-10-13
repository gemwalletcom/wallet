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
        let Some(name) = ens_name(query) else {
            return Ok(None);
        };
        let resolver = self.client.get_resolver(&name).await?;
        if resolver.is_zero() {
            return Ok(None);
        }
        let address = self.client.get_address(&resolver, &name).await?;
        Ok((!address.is_zero()).then(|| address.to_string()))
    }
}

fn ens_name(query: &NameQuery) -> Option<String> {
    query.ascii_domain().ok().filter(|name| !name.split('.').any(|label| label.starts_with("xn--")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ens_name() {
        assert_eq!(ens_name(&NameQuery::new("Vitalik.ETH")).as_deref(), Some("vitalik.eth"));
        assert_eq!(ens_name(&NameQuery::new("🐈.eth")), None);
        assert_eq!(ens_name(&NameQuery::new("xn--zn8h.eth")), None);
        assert_eq!(ens_name(&NameQuery::new("mañana.eth")), None);
    }
}
