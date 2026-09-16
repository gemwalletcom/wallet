use std::collections::HashMap;
use std::sync::Arc;

use primitives::Chain;

use super::model::{GemAddNodeError, GemChainSettingsSection, GemExplorerRow, GemNodeCheck, GemNodeRow, GemNodeSelection, GemNodeStatusState};
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

    pub fn sections(&self) -> Vec<GemChainSettingsSection> {
        vec![GemChainSettingsSection::Nodes, GemChainSettingsSection::Explorer]
    }

    pub fn explorers(&self, chain: Chain) -> Vec<String> {
        self.explorer.get_explorers(chain)
    }

    pub fn explorer_rows(&self, chain: Chain) -> Vec<GemExplorerRow> {
        let selected = self.explorer.get_explorer_name(chain);
        self.explorer
            .get_explorers(chain)
            .into_iter()
            .map(|name| GemExplorerRow {
                is_selected: name == selected,
                name,
            })
            .collect()
    }

    pub fn node_rows(&self, chain: Chain, nodes: Vec<GemNodeSelection>, statuses: HashMap<String, GemNodeStatusState>) -> Vec<GemNodeRow> {
        nodes
            .into_iter()
            .map(|node| {
                let status = statuses.get(&node.url).cloned().unwrap_or(GemNodeStatusState::Loading);
                self.node_row(chain, node, status)
            })
            .collect()
    }

    pub fn node_row(&self, chain: Chain, node: GemNodeSelection, status: GemNodeStatusState) -> GemNodeRow {
        GemNodeRow {
            title: node.title(),
            subtitle: status.subtitle(),
            latency_status: status.latency_status(),
            can_delete: self.can_delete_node(chain, node.url.clone()),
            node,
        }
    }

    pub fn explorer_name(&self, chain: Chain) -> String {
        self.explorer.get_explorer_name(chain)
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

    pub async fn node_status(&self, chain: Chain, url: String) -> GemNodeStatusState {
        rules::node_status_state(self.gateway.get_node_status(chain, &url).await.ok())
    }

    pub fn new_node_list_session(&self, chain: Chain) -> GemNodeListSession {
        GemNodeListSession::new(chain)
    }

    pub fn new_add_node_session(&self, chain: Chain) -> GemAddNodeSession {
        GemAddNodeSession::new(chain)
    }

    pub fn node_check_debounce_milliseconds(&self) -> u64 {
        rules::node_check_debounce_milliseconds()
    }

    pub async fn check_node(&self, chain: Chain, url: String) -> Result<GemNodeCheck, GemAddNodeError> {
        let url = rules::node_url(&url).ok_or(GemAddNodeError::InvalidUrl)?;
        Ok(self.gateway.check_node(chain, &url).await?)
    }
}

#[cfg(test)]
mod tests {
    use primitives::node_config::NodeRegion;

    use super::super::model::GemNodeSubtitle;
    use super::*;
    use crate::gateway::EmptyPreferences;
    use crate::services::node::rules;
    use crate::services::node::testkit::MemoryNodeStore;
    use crate::services::preferences::GemPreferencesService;
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::testkit::TestAlienProvider;

    fn service() -> GemChainSettingsService {
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        GemChainSettingsService::new(
            Arc::new(GemNodeService::new(Arc::new(MemoryNodeStore::default()), preferences_store.clone())),
            Arc::new(GemExplorerService::new(preferences)),
            Arc::new(GemGateway::new(
                Arc::new(TestAlienProvider::with_status(200)),
                preferences_store,
                Arc::new(EmptyPreferences),
            )),
        )
    }

    #[test]
    fn test_exactly_one_explorer_row_is_selected_and_it_follows_the_stored_name() {
        let service = service();
        let rows = service.explorer_rows(Chain::Ethereum);
        let names: Vec<String> = rows.iter().map(|row| row.name.clone()).collect();

        assert_eq!(names, service.explorers(Chain::Ethereum));
        assert_eq!(rows.iter().filter(|row| row.is_selected).count(), 1);

        let other = names.last().unwrap().clone();
        service.set_explorer_name(Chain::Ethereum, other.clone()).unwrap();
        let selected: Vec<String> = service
            .explorer_rows(Chain::Ethereum)
            .into_iter()
            .filter(|row| row.is_selected)
            .map(|row| row.name)
            .collect();

        assert_eq!(selected, vec![other]);
    }

    #[test]
    fn test_node_rows_pair_each_node_with_its_own_status_and_defaults_the_rest_to_loading() {
        let service = service();
        let default_url = rules::region_node(Chain::Ethereum, NodeRegion::Us).url;
        let selections = rules::node_selections(vec![rules::region_node(Chain::Ethereum, NodeRegion::Us)], &default_url);
        let added = GemNodeSelection {
            url: "https://node.example.com".to_string(),
            host: "node.example.com".to_string(),
            is_selected: false,
            gem_node_flag: None,
        };
        let nodes = vec![selections[0].clone(), added.clone()];
        let statuses = HashMap::from([(
            added.url.clone(),
            GemNodeStatusState::Result {
                latest_block_number: 21_000_000,
                latency: primitives::Latency::from_milliseconds(120),
            },
        )]);

        let rows = service.node_rows(Chain::Ethereum, nodes, statuses);

        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].subtitle,
            GemNodeSubtitle::LatestBlock { value: "-".to_string() },
            "a node with no status yet is still loading"
        );
        assert_eq!(rows[1].subtitle, GemNodeSubtitle::LatestBlock { value: "21,000,000".to_string() });
        assert!(!rows[0].can_delete);
        assert!(rows[1].can_delete);
    }

    #[test]
    fn test_a_default_node_row_cannot_be_deleted_and_an_added_one_can() {
        let service = service();
        let default_url = rules::region_node(Chain::Ethereum, NodeRegion::Us).url;
        let selections = rules::node_selections(vec![rules::region_node(Chain::Ethereum, NodeRegion::Us)], &default_url);
        let default_row = service.node_row(Chain::Ethereum, selections[0].clone(), GemNodeStatusState::Loading);

        assert!(!default_row.can_delete);
        assert_eq!(default_row.subtitle, GemNodeSubtitle::LatestBlock { value: "-".to_string() });

        let added = GemNodeSelection {
            url: "https://node.example.com".to_string(),
            host: "node.example.com".to_string(),
            is_selected: false,
            gem_node_flag: None,
        };
        let added_row = service.node_row(
            Chain::Ethereum,
            added,
            GemNodeStatusState::Result {
                latest_block_number: 21_000_000,
                latency: primitives::Latency::from_milliseconds(120),
            },
        );

        assert!(added_row.can_delete);
        assert_eq!(added_row.subtitle, GemNodeSubtitle::LatestBlock { value: "21,000,000".to_string() });
    }
}
