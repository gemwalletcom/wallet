use std::error::Error;

use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, NameProvider};

use super::client::IcnsClient;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

const DOMAINS: &[(&str, Chain)] = &[("cosmos", Chain::Cosmos), ("osmo", Chain::Osmosis), ("celestia", Chain::Celestia), ("sei", Chain::Sei)];

pub struct IcnsProvider {
    client: IcnsClient,
}

impl IcnsProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: IcnsClient::new(client) }
    }
}

#[async_trait]
impl NameResolver for IcnsProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Icns
    }

    fn domains(&self) -> Vec<&'static str> {
        DOMAINS.iter().map(|(domain, _)| *domain).collect()
    }

    fn chains(&self) -> Vec<Chain> {
        DOMAINS.iter().map(|(_, chain)| *chain).collect()
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let suffix_chain = DOMAINS
            .iter()
            .find_map(|(domain, chain)| (*domain == query.suffix).then_some(*chain))
            .ok_or(format!("unsupported domain: {}", query.suffix))?;
        if suffix_chain != chain {
            return Err(format!("domain {} does not match chain {chain}", query.suffix).into());
        }
        Ok(Some(self.client.get_record(&query.domain).await?.bech32_address))
    }
}
