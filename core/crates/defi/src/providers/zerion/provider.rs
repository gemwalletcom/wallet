use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use primitives::{Chain, DefiPosition};

use crate::provider::DefiProvider as DefiProviderTrait;

use super::client::ZerionClient;
use super::mapper::map_positions;

#[async_trait]
impl<C: Client> DefiProviderTrait for ZerionClient<C> {
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

    async fn get_positions(&self, chain: Chain, address: &str) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        map_positions(self.get_wallet_positions(chain, address).await?, chain)
    }
}
