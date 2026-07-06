use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, DefiPosition};

use crate::provider::DefiProvider as DefiProviderTrait;

use super::{JupiterClient, mapper::map_positions};

#[async_trait]
impl DefiProviderTrait for JupiterClient {
    fn chains(&self) -> &'static [Chain] {
        &[Chain::Solana]
    }

    async fn get_positions(&self, _chain: Chain, address: &str) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        map_positions(self.get_wallet_positions(address).await?)
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use crate::provider::DefiProvider;

    use super::JupiterClient;

    #[test]
    fn test_chains() {
        let client = JupiterClient::new_with_api_key("https://api.jup.ag".to_string(), String::new());

        assert_eq!(client.chains(), &[Chain::Solana]);
    }
}
