use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use primitives::{Chain, DefiPosition};

#[async_trait]
pub trait DefiProvider: Send + Sync {
    fn chains(&self) -> &'static [Chain];

    async fn get_positions(&self, chain: Chain, address: &str) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>>;
}

pub struct DefiProviders {
    providers: Vec<Arc<dyn DefiProvider>>,
}

impl DefiProviders {
    pub fn new(providers: Vec<Arc<dyn DefiProvider>>) -> Self {
        Self { providers }
    }

    fn providers_for_chain(&self, chain: Chain) -> impl Iterator<Item = &Arc<dyn DefiProvider>> {
        self.providers.iter().filter(move |provider| provider.chains().contains(&chain))
    }

    pub async fn get_positions(&self, chain: Chain, address: &str) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        let futures = self.providers_for_chain(chain).map(|provider| provider.get_positions(chain, address));
        let results = futures::future::join_all(futures).await;

        Ok(results.into_iter().filter_map(Result::ok).flatten().collect())
    }
}
