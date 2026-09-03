use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainState;

use crate::rpc::{EthereumClient, EthereumProvider};
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::NodeSyncStatus;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainState for EthereumProvider<C> {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        Ok(Some(EthereumClient::get_chain_id(self).await?.to_string()))
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let latest_block = self.get_block_latest_number().await?;
        Ok(NodeSyncStatus::synced(latest_block))
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_latest_block().await
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use chain_traits::ChainState;
    use primitives::Chain;

    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client};

    #[tokio::test]
    async fn test_ethereum_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let chain_id = ChainState::get_chain_id(&client).await?;

        assert_eq!(chain_id.as_deref(), Some(Chain::Ethereum.network_id()));

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let block_number = ChainState::get_block_latest_number(&client).await?;

        println!("Ethereum Latest Block: {}", block_number);

        assert!(block_number > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let chain_id = ChainState::get_chain_id(&client).await?;

        assert_eq!(chain_id.as_deref(), Some(Chain::SmartChain.network_id()));

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let block_number = ChainState::get_block_latest_number(&client).await?;

        println!("SmartChain Latest Block: {}", block_number);

        assert!(block_number > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let node_status = ChainState::get_node_status(&client).await?;

        println!("Ethereum Node Status: {:?}", node_status);

        assert!(node_status.in_sync);

        Ok(())
    }
}
