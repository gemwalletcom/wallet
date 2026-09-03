use std::sync::Arc;

use primitives::node::Node;
use primitives::node_config::NodeRegion;
use primitives::{Chain, Latency};

use super::model::{GemAddNodeError, GemNodeCheck, GemNodeStatusState};
use super::rules;
use crate::gateway::GemGateway;
use crate::services::chain::rules as chain_rules;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::node::GemNodeService;

#[derive(uniffi::Object)]
pub struct GemChainSettingsService {
    nodes: Arc<GemNodeService>,
    explorer: Arc<GemExplorerService>,
    gateway: Arc<GemGateway>,
}

#[uniffi::export]
impl GemChainSettingsService {
    #[uniffi::constructor]
    pub fn new(nodes: Arc<GemNodeService>, explorer: Arc<GemExplorerService>, gateway: Arc<GemGateway>) -> Self {
        Self { nodes, explorer, gateway }
    }

    pub fn chains(&self, query: String) -> Vec<Chain> {
        chain_rules::matching_chains(chain_rules::chains_by_rank(), &query)
    }

    pub fn explorers(&self, chain: Chain) -> Vec<String> {
        self.explorer.get_explorers(chain)
    }

    pub fn explorer_name(&self, chain: Chain) -> String {
        self.explorer.get_explorer_name(chain)
    }

    pub fn set_explorer_name(&self, chain: Chain, name: String) -> Result<(), GemServiceError> {
        self.explorer.set_explorer_name(chain, name)
    }

    pub async fn nodes(&self, chain: Chain) -> Result<Vec<Node>, GemServiceError> {
        Ok(self.nodes.sorted_nodes(chain, self.nodes.get_nodes(chain).await?))
    }

    pub fn selected_node(&self, chain: Chain) -> Node {
        self.nodes.selected_node(chain)
    }

    pub async fn select_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.select_node(chain, url).await
    }

    pub fn can_delete_node(&self, chain: Chain, url: String) -> bool {
        self.nodes.can_delete_node(chain, url)
    }

    pub async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.delete_node(chain, url).await
    }

    pub async fn add_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.add_node(chain, url.clone()).await?;
        self.nodes.select_node(chain, url).await
    }

    pub fn node_flag(&self, url: String) -> Option<String> {
        NodeRegion::from_url(&url).map(|region| region.flag().to_string())
    }

    pub async fn node_status(&self, chain: Chain, url: String) -> GemNodeStatusState {
        match self.gateway.get_node_status(chain, &url).await {
            Ok(status) if status.latest_block_number > 0 => GemNodeStatusState::Result {
                latest_block_number: status.latest_block_number,
                latency: Latency::from_milliseconds(status.latency_ms),
            },
            Ok(_) | Err(_) => GemNodeStatusState::Error,
        }
    }

    pub fn node_check_debounce_milliseconds(&self) -> u64 {
        rules::node_check_debounce_milliseconds()
    }

    pub async fn check_node(&self, chain: Chain, url: String) -> Result<GemNodeCheck, GemAddNodeError> {
        let url = rules::node_url(&url).ok_or(GemAddNodeError::InvalidUrl)?;
        Ok(self.gateway.check_node(chain, &url).await?)
    }
}
