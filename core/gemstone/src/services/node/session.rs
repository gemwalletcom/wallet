use std::collections::HashMap;

use primitives::Chain;

use super::model::{GemAddNodeError, GemChainSettingsSection, GemExplorerRow, GemNodeCheck, GemNodeRow, GemNodeSelection, GemNodeStatusState};
use super::rules;
use crate::services::error::GemServiceError;
use crate::services::error_text::GemErrorText;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAddNodePhase {
    Idle,
    Checking,
    Ready { check: GemNodeCheck },
    Failed { error: GemErrorText },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddNodeViewState {
    pub phase: GemAddNodePhase,
    pub can_import: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddNodeSession {
    pub chain: Chain,
    pub url: String,
    pub check: Option<GemNodeCheck>,
    pub error: Option<GemErrorText>,
    pub is_checking: bool,
}

impl GemAddNodeSession {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            url: String::new(),
            check: None,
            error: None,
            is_checking: false,
        }
    }
}

#[uniffi::export]
impl GemAddNodeSession {
    pub fn on_input(&self, url: String) -> Self {
        Self {
            url: url.trim().to_string(),
            check: None,
            error: None,
            is_checking: false,
            ..self.clone()
        }
    }

    pub fn on_checking(&self) -> Self {
        Self {
            check: None,
            error: None,
            is_checking: !self.url.is_empty(),
            ..self.clone()
        }
    }

    pub fn on_checked(&self, url: String, check: GemNodeCheck) -> Self {
        if url != self.url {
            return self.clone();
        }
        Self {
            check: Some(check),
            error: None,
            is_checking: false,
            ..self.clone()
        }
    }

    pub fn on_check_failed(&self, url: String, error: Option<GemAddNodeError>) -> Self {
        if url != self.url {
            return self.clone();
        }
        self.failed(error.map(|error| error.text()))
    }

    pub fn on_add_failed(&self, error: Option<GemServiceError>) -> Self {
        self.failed(error.map(|error| error.text()))
    }

    pub fn on_imported(&self) -> Self {
        Self::new(self.chain)
    }

    pub fn checks_url(&self) -> bool {
        !self.url.is_empty()
    }

    pub fn view_state(&self) -> GemAddNodeViewState {
        GemAddNodeViewState {
            phase: self.phase(),
            can_import: self.check.is_some(),
        }
    }
}

impl GemAddNodeSession {
    fn failed(&self, error: Option<GemErrorText>) -> Self {
        Self {
            check: None,
            error: Some(error.unwrap_or(GemErrorText::Unknown)),
            is_checking: false,
            ..self.clone()
        }
    }

    fn phase(&self) -> GemAddNodePhase {
        if self.is_checking {
            return GemAddNodePhase::Checking;
        }
        match (&self.check, &self.error) {
            (Some(check), _) => GemAddNodePhase::Ready { check: check.clone() },
            (None, Some(error)) => GemAddNodePhase::Failed { error: error.clone() },
            (None, None) => GemAddNodePhase::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::GatewayError;

    #[test]
    fn test_an_empty_url_is_idle_and_never_starts_a_check() {
        let session = GemAddNodeSession::new(Chain::Ethereum).on_input("   ".to_string());

        assert!(!session.checks_url());
        assert_eq!(session.on_checking().view_state().phase, GemAddNodePhase::Idle);
    }

    #[test]
    fn test_a_new_url_clears_the_previous_answer() {
        let checked = GemAddNodeSession::new(Chain::Ethereum).on_input("https://node".to_string()).on_checked("https://node".to_string(), GemNodeCheck::mock());
        assert!(checked.view_state().can_import);

        let retyped = checked.on_input("https://other".to_string());
        assert_eq!(retyped.view_state().phase, GemAddNodePhase::Idle);
        assert!(!retyped.view_state().can_import, "a url that was never checked cannot be imported");
    }

    #[test]
    fn test_a_check_for_an_earlier_url_is_ignored() {
        let session = GemAddNodeSession::new(Chain::Ethereum).on_input("https://other".to_string()).on_checking();

        assert_eq!(session.on_checked("https://node".to_string(), GemNodeCheck::mock()), session);
        assert_eq!(session.on_check_failed("https://node".to_string(), Some(GemAddNodeError::InvalidUrl)), session);
    }

    #[test]
    fn test_a_failure_replaces_the_answer_and_blocks_the_import() {
        let failed = GemAddNodeSession::new(Chain::Ethereum)
            .on_input("https://node".to_string())
            .on_checked("https://node".to_string(), GemNodeCheck::mock())
            .on_check_failed("https://node".to_string(), Some(GemAddNodeError::InvalidNetworkId));

        assert_eq!(failed.view_state().phase, GemAddNodePhase::Failed { error: GemErrorText::InvalidNetworkId });
        assert!(!failed.view_state().can_import);
    }

    #[test]
    fn test_a_failure_carries_the_error_the_apps_localize() {
        let session = GemAddNodeSession::new(Chain::Ethereum).on_input("https://node".to_string()).on_checking();

        assert_eq!(
            session.clone().on_check_failed("https://node".to_string(), Some(GemAddNodeError::Gateway(GatewayError::Offline))).error,
            Some(GemErrorText::NetworkOffline),
            "a transport failure does not pretend to be a bad url"
        );
        assert_eq!(
            session.clone().on_add_failed(Some(GemServiceError::Store { msg: "disk full".to_string() })).error,
            Some(GemErrorText::Unknown),
            "storage text is internal"
        );
        assert_eq!(session.on_check_failed("https://node".to_string(), None).error, Some(GemErrorText::Unknown));
    }

    #[test]
    fn test_importing_leaves_the_screen_ready_for_the_next_url() {
        let imported = GemAddNodeSession::new(Chain::Ethereum)
            .on_input("https://node".to_string())
            .on_checked("https://node".to_string(), GemNodeCheck::mock())
            .on_imported();

        assert_eq!(imported, GemAddNodeSession::new(Chain::Ethereum));
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNodeListSession {
    pub chain: Chain,
    pub nodes: Vec<GemNodeSelection>,
    pub statuses: HashMap<String, GemNodeStatusState>,
}

impl GemNodeListSession {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            nodes: Vec::new(),
            statuses: HashMap::new(),
        }
    }

    pub fn rows(&self) -> Vec<GemNodeRow> {
        self.nodes
            .iter()
            .map(|node| {
                let status = self.statuses.get(&node.url).cloned().unwrap_or(GemNodeStatusState::Loading);
                GemNodeRow {
                    title: node.title(),
                    subtitle: status.subtitle(),
                    latency_status: status.latency_status(),
                    can_delete: rules::can_delete_node(self.chain, &node.url),
                    node: node.clone(),
                }
            })
            .collect()
    }
}

#[uniffi::export]
impl GemNodeListSession {
    pub fn on_nodes(&self, nodes: Vec<GemNodeSelection>) -> Self {
        Self {
            statuses: rules::visible_statuses(&nodes, &self.statuses),
            nodes,
            ..self.clone()
        }
    }

    pub fn on_checking(&self) -> Self {
        Self {
            statuses: self.nodes.iter().map(|node| (node.url.clone(), GemNodeStatusState::Loading)).collect(),
            ..self.clone()
        }
    }

    pub fn on_status(&self, url: String, state: GemNodeStatusState) -> Self {
        if !self.nodes.iter().any(|node| node.url == url) {
            return self.clone();
        }
        let mut statuses = self.statuses.clone();
        statuses.insert(url, state);
        Self { statuses, ..self.clone() }
    }

    pub fn node_urls(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.url.clone()).collect()
    }

    pub fn sections(&self, explorers: Vec<GemExplorerRow>) -> Vec<GemChainSettingsSection> {
        vec![GemChainSettingsSection::Nodes { rows: self.rows() }, GemChainSettingsSection::Explorers { rows: explorers }]
    }
}

#[cfg(test)]
mod node_list_tests {
    use primitives::node_config::NodeRegion;

    use super::super::model::GemNodeSubtitle;
    use super::*;
    use crate::services::node::rules;

    #[test]
    fn test_a_status_for_a_node_that_is_gone_is_dropped() {
        let session = GemNodeListSession::new(Chain::Ethereum)
            .on_nodes(vec![GemNodeSelection::mock("a"), GemNodeSelection::mock("b")])
            .on_status("a".to_string(), GemNodeStatusState::mock_result(1))
            .on_status("b".to_string(), GemNodeStatusState::mock_result(2));

        let after_delete = session.on_nodes(vec![GemNodeSelection::mock("a")]);

        assert_eq!(after_delete.statuses.len(), 1);
        assert!(after_delete.statuses.contains_key("a"));
    }

    #[test]
    fn test_a_status_that_arrives_for_a_node_the_list_no_longer_has_is_ignored() {
        let session = GemNodeListSession::new(Chain::Ethereum).on_nodes(vec![GemNodeSelection::mock("a")]);

        let late = session.on_status("gone".to_string(), GemNodeStatusState::mock_result(3));

        assert!(late.statuses.is_empty(), "a late answer must not add a row back");
    }

    #[test]
    fn test_checking_puts_every_node_back_to_loading() {
        let session = GemNodeListSession::new(Chain::Ethereum)
            .on_nodes(vec![GemNodeSelection::mock("a"), GemNodeSelection::mock("b")])
            .on_status("a".to_string(), GemNodeStatusState::mock_result(1))
            .on_checking();

        assert_eq!(session.node_urls(), vec!["a".to_string(), "b".to_string()]);
        assert!(session.statuses.values().all(|state| *state == GemNodeStatusState::Loading));
    }
    #[test]
    fn test_node_rows_pair_each_node_with_its_own_status_and_defaults_the_rest_to_loading() {
        let default_url = rules::region_node(Chain::Ethereum, NodeRegion::Us).url;
        let selections = rules::node_selections(vec![rules::region_node(Chain::Ethereum, NodeRegion::Us)], &default_url);
        let added = GemNodeSelection {
            host: "node.example.com".to_string(),
            ..GemNodeSelection::mock("https://node.example.com")
        };
        let nodes = vec![selections[0].clone(), added.clone()];
        let statuses = HashMap::from([(added.url.clone(), GemNodeStatusState::mock_result(21_000_000))]);

        let rows = GemNodeListSession { chain: Chain::Ethereum, nodes, statuses }.rows();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].subtitle, GemNodeSubtitle::LatestBlock { value: None }, "a node with no status yet is still loading");
        assert_eq!(
            rows[1].subtitle,
            GemNodeSubtitle::LatestBlock {
                value: Some(crate::formatted_number::GemFormattedNumber::count(21_000_000))
            }
        );
        assert!(!rows[0].can_delete);
        assert!(rows[1].can_delete);
    }

    #[test]
    fn test_the_settings_list_the_nodes_before_the_explorers() {
        let session = GemNodeListSession::new(Chain::Ethereum).on_nodes(vec![GemNodeSelection::mock("a")]);
        let explorers = vec![GemExplorerRow {
            name: "Etherscan".to_string(),
            is_selected: true,
        }];

        assert_eq!(
            session.sections(explorers.clone()),
            vec![GemChainSettingsSection::Nodes { rows: session.rows() }, GemChainSettingsSection::Explorers { rows: explorers }]
        );
    }
}
