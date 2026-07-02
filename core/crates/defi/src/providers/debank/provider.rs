use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, DefiPosition};

use crate::provider::DefiProvider as DefiProviderTrait;

use super::client::DeBankClient;

#[async_trait]
impl DefiProviderTrait for DeBankClient {
    fn chains(&self) -> &'static [Chain] {
        &[
            Chain::Ethereum,
            Chain::SmartChain,
            Chain::Polygon,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Base,
            Chain::AvalancheC,
            Chain::Fantom,
            Chain::Gnosis,
            Chain::ZkSync,
            Chain::Linea,
            Chain::Celo,
        ]
    }

    async fn get_positions(&self, _chain: Chain, _address: &str) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        Err("DeBank DeFi positions provider is not implemented".into())
    }
}
