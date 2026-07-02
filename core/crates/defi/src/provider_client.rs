use std::error::Error;

use primitives::{Chain, DefiPosition};

use crate::config::DefiProviderConfig;
use crate::factory::DefiProviderFactory;
use crate::provider::DefiProviders;

pub struct DefiProviderClient {
    providers: DefiProviders,
}

impl DefiProviderClient {
    pub fn new(config: DefiProviderConfig) -> Self {
        Self {
            providers: DefiProviders::new(DefiProviderFactory::new_providers(config)),
        }
    }

    pub async fn get_positions(&self, chain: Chain, address: &str) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        self.providers.get_positions(chain, address).await
    }
}
