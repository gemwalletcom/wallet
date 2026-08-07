use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::rpc::PolkadotProvider;

#[async_trait]
impl<C: Client> ChainState for PolkadotProvider<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        Ok(self.get_node_version().await?.chain)
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_block_header("head").await?.number)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let latest_block = self.get_block_latest_number().await?;
        Ok(NodeSyncStatus::synced(latest_block))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::{PolkadotClient, PolkadotProvider};
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rpc_client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/v1/blocks/head/header");
            Ok(r#"{"number":"123456"}"#.as_bytes().to_vec())
        });
        let client = PolkadotProvider::new_rpc_only(PolkadotClient::new(rpc_client));

        let status = client.get_node_status().await?;

        assert!(status.in_sync);
        assert_eq!(status.latest_block_number, Some(123456));
        assert_eq!(status.current_block_number, Some(123456));

        Ok(())
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_polkadot_test_client;

    #[tokio::test]
    async fn test_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let chain_id = client.get_chain_id().await?;
        assert!(!chain_id.is_empty());
        println!("Chain ID: {}", chain_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_polkadot_test_client();
        let block_number = client.get_block_latest_number().await?;
        assert!(block_number > 0);
        println!("Latest block: {}", block_number);
        Ok(())
    }
}
