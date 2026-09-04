use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainState;
use gem_client::Client;
use primitives::NodeSyncStatus;

use crate::provider::state_mapper;
use crate::rpc::XrpClient;

#[async_trait]
impl<C: Client + Clone> ChainState for XrpClient<C> {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        Ok(self.get_server_info().await?.network_id.map(|network_id| network_id.to_string()))
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_latest_validated_ledger().await?.ledger_index)
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        let ledger_info = self.get_latest_validated_ledger().await?;
        state_mapper::map_node_status(&ledger_info)
    }
}

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::Chain;
    use serde_json::json;

    use super::*;
    use crate::method;

    #[tokio::test]
    async fn test_get_chain_id_reads_the_network_id_and_stays_unset_when_the_server_omits_it() {
        let client = XrpClient::new(mock_jsonrpc_client(|rpc_method, params| {
            assert_eq!(rpc_method, method::SERVER_INFO);
            assert_eq!(params, &json!([{}]));
            Ok(json!({"info": {"network_id": 0}, "status": "success"}))
        }));
        assert_eq!(client.get_chain_id().await.unwrap().as_deref(), Some(Chain::Xrp.network_id()));

        let client = XrpClient::new(mock_jsonrpc_client(|_, _| Ok(json!({"info": {}, "status": "success"}))));
        assert_eq!(client.get_chain_id().await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_get_block_latest_number_uses_validated_ledger() {
        let client = XrpClient::new(mock_jsonrpc_client(|rpc_method, params| {
            assert_eq!(rpc_method, method::LEDGER);
            assert_eq!(params, &json!([{"ledger_index": "validated"}]));
            Ok(json!({
                "ledger_index": 80123456,
                "validated": true,
                "status": "success"
            }))
        }));
        assert_eq!(client.get_block_latest_number().await.unwrap(), 80123456);

        let client = XrpClient::new(mock_jsonrpc_client(|_, _| {
            Ok(json!({
                "ledger_index": 80123457,
                "validated": false,
                "status": "success"
            }))
        }));
        assert_eq!(client.get_block_latest_number().await.unwrap_err().to_string(), "XRP RPC returned an unvalidated ledger");
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use primitives::Chain;

    use super::*;
    use crate::provider::testkit::create_xrp_test_client;

    #[tokio::test]
    async fn test_get_xrp_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let chain_id = client.get_chain_id().await?;

        assert_eq!(chain_id.as_deref(), Some(Chain::Xrp.network_id()));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_xrp_latest_validated_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let block_number = client.get_block_latest_number().await?;
        let ledger = client.get_block_transactions(block_number).await?;

        assert!(block_number > 80_000_000, "XRP ledger index should be above 80M, got: {}", block_number);
        assert_eq!(ledger.ledger_index, block_number);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_node_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let node_status = client.get_node_status().await?;

        assert!(node_status.in_sync);
        assert!(node_status.latest_block_number.is_some());
        assert!(node_status.latest_block_number.unwrap_or(0) > 80_000_000);

        Ok(())
    }
}
