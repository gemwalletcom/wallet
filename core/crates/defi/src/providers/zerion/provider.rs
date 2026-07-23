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

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use crate::provider::DefiProvider;

    use super::ZerionClient;
    use gem_client::ReqwestClient;

    #[test]
    fn test_chains() {
        let client = ZerionClient::new(ReqwestClient::new(String::new(), gem_client::reqwest_client()));

        assert!(client.chains().contains(&Chain::Ethereum));
        assert!(client.chains().contains(&Chain::Base));
        assert!(!client.chains().contains(&Chain::Solana));
    }
}
