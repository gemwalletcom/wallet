use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainState for HyperCoreClient<C> {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(1)
    }
}
