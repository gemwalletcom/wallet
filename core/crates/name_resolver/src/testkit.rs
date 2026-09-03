use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, NameProvider};

use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct MockNameResolver {
    provider: NameProvider,
    domains: Vec<&'static str>,
    chains: Vec<Chain>,
    response: Result<&'static str, &'static str>,
}

impl MockNameResolver {
    pub fn new(provider: NameProvider, domains: Vec<&'static str>, chains: Vec<Chain>, response: Result<&'static str, &'static str>) -> Self {
        Self {
            provider,
            domains,
            chains,
            response,
        }
    }
}

#[async_trait]
impl NameResolver for MockNameResolver {
    fn provider(&self) -> NameProvider {
        self.provider.clone()
    }

    fn domains(&self) -> Vec<&'static str> {
        self.domains.clone()
    }

    fn chains(&self) -> Vec<Chain> {
        self.chains.clone()
    }

    async fn resolve(&self, _query: &NameQuery, _chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        match self.response {
            Ok(address) => Ok(Some(address.to_string())),
            Err(error) => Err(error.into()),
        }
    }
}
