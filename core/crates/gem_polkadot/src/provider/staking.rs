use async_trait::async_trait;
use chain_traits::ChainStaking;
use std::error::Error;

use gem_client::Client;

use crate::rpc::PolkadotProvider;

#[async_trait]
impl<C: Client> ChainStaking for PolkadotProvider<C> {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(Some(10.0))
    }
}
