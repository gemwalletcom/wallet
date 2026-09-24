use std::sync::Arc;

use primitives::Chain;

use super::model::{GemAddNodeError, GemExplorerRow, GemNodeCheck, GemNodeSelection, GemNodeStatusState};
use super::rules;
use super::session::{GemAddNodeSession, GemNodeListSession};
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

    pub fn explorer_rows(&self, chain: Chain) -> Vec<GemExplorerRow> {
        let selected = self.explorer.get_explorer_name(chain);
        self.explorer.get_explorers(chain).into_iter().map(|name| GemExplorerRow { is_selected: name == selected, name }).collect()
    }

    pub fn set_explorer_name(&self, chain: Chain, name: String) -> Result<(), GemServiceError> {
        self.explorer.set_explorer_name(chain, name)
    }

    pub async fn nodes(&self, chain: Chain) -> Result<Vec<GemNodeSelection>, GemServiceError> {
        let nodes = self.nodes.get_nodes(chain).await?;
        let selected_url = self.nodes.selected_node(chain).url;
        Ok(rules::node_selections(self.nodes.sorted_nodes(chain, nodes), &selected_url))
    }

    pub async fn select_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.select_node(chain, url).await
    }

    pub async fn delete_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.delete_node(chain, url).await
    }

    pub async fn add_node(&self, chain: Chain, url: String) -> Result<(), GemServiceError> {
        self.nodes.add_node(chain, url.clone()).await?;
        self.nodes.select_node(chain, url).await
    }

    pub async fn node_status(&self, chain: Chain, url: String) -> GemNodeStatusState {
        rules::node_status_state(self.gateway.get_node_status(chain, &url).await.ok())
    }

    pub fn new_node_list_session(&self, chain: Chain) -> GemNodeListSession {
        GemNodeListSession::new(chain)
    }

    pub fn new_add_node_session(&self, chain: Chain) -> GemAddNodeSession {
        GemAddNodeSession::new(chain)
    }

    pub async fn check_node(&self, chain: Chain, url: String) -> Result<GemNodeCheck, GemAddNodeError> {
        let url = rules::node_url(&url).ok_or(GemAddNodeError::InvalidUrl)?;
        Ok(self.gateway.check_node(chain, &url).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exactly_one_explorer_row_is_selected_and_it_follows_the_stored_name() {
        let service = GemChainSettingsService::mock();
        let rows = service.explorer_rows(Chain::Ethereum);
        let names: Vec<String> = rows.iter().map(|row| row.name.clone()).collect();

        assert_eq!(names, service.explorer.get_explorers(Chain::Ethereum));
        assert_eq!(rows.iter().filter(|row| row.is_selected).count(), 1);

        let other = names.last().unwrap().clone();
        service.set_explorer_name(Chain::Ethereum, other.clone()).unwrap();
        let selected: Vec<String> = service.explorer_rows(Chain::Ethereum).into_iter().filter(|row| row.is_selected).map(|row| row.name).collect();

        assert_eq!(selected, vec![other]);
    }
}
