use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::NearProvider;

#[async_trait]
impl<C: Client + Clone> ChainState for NearProvider<C> {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        Ok(Some(self.get_status().await?.chain_id))
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_latest_block().await?.header.height)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let block = self.get_latest_block().await?;
        state_mapper::map_node_status(&block)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use std::error::Error;

    use chain_traits::{ChainProvider, ChainState};
    use primitives::Chain;

    use crate::provider::testkit::create_near_test_client;

    #[tokio::test]
    async fn test_near_client_generic_interface() {
        let near_client = create_near_test_client();

        assert_eq!(near_client.get_chain().to_string(), "near");
    }

    #[tokio::test]
    async fn test_get_chain_id() -> Result<(), Box<dyn Error + Send + Sync>> {
        let near_client = create_near_test_client();

        let chain_id = near_client.get_chain_id().await?;
        assert_eq!(chain_id.as_deref(), Some(Chain::Near.network_id()));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn Error + Send + Sync>> {
        let near_client = create_near_test_client();
        let node_status = near_client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 0);

        Ok(())
    }
}
