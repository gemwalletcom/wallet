use std::collections::HashMap;

use primitives::Chain;

use super::model::{GemNodeCheck, GemNodeSelection, GemNodeStatusState};
use super::rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAddNodeFailure {
    InvalidUrl,
    InvalidNetworkId,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAddNodePhase {
    Idle,
    Checking,
    Ready { check: GemNodeCheck },
    Failed { failure: GemAddNodeFailure },
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
    pub failure: Option<GemAddNodeFailure>,
    pub is_checking: bool,
}

impl GemAddNodeSession {
    pub fn new(chain: Chain) -> Self {
        Self {
            chain,
            url: String::new(),
            check: None,
            failure: None,
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
            failure: None,
            is_checking: false,
            ..self.clone()
        }
    }

    pub fn on_checking(&self) -> Self {
        Self {
            check: None,
            failure: None,
            is_checking: !self.url.is_empty(),
            ..self.clone()
        }
    }

    pub fn on_checked(&self, check: GemNodeCheck) -> Self {
        Self {
            check: Some(check),
            failure: None,
            is_checking: false,
            ..self.clone()
        }
    }

    pub fn on_failed(&self, failure: GemAddNodeFailure) -> Self {
        Self {
            check: None,
            failure: Some(failure),
            is_checking: false,
            ..self.clone()
        }
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
    fn phase(&self) -> GemAddNodePhase {
        if self.is_checking {
            return GemAddNodePhase::Checking;
        }
        match (&self.check, self.failure) {
            (Some(check), _) => GemAddNodePhase::Ready { check: check.clone() },
            (None, Some(failure)) => GemAddNodePhase::Failed { failure },
            (None, None) => GemAddNodePhase::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Latency;

    fn check() -> GemNodeCheck {
        GemNodeCheck {
            url: "https://node".to_string(),
            chain_id: None,
            latest_block_number: 1,
            is_in_sync: true,
            latency: Latency::from_milliseconds(10),
        }
    }

    #[test]
    fn test_an_empty_url_is_idle_and_never_starts_a_check() {
        let session = GemAddNodeSession::new(Chain::Ethereum).on_input("   ".to_string());

        assert!(!session.checks_url());
        assert_eq!(session.on_checking().view_state().phase, GemAddNodePhase::Idle);
    }

    #[test]
    fn test_a_new_url_clears_the_previous_answer() {
        let checked = GemAddNodeSession::new(Chain::Ethereum).on_input("https://node".to_string()).on_checked(check());
        assert!(checked.view_state().can_import);

        let retyped = checked.on_input("https://other".to_string());
        assert_eq!(retyped.view_state().phase, GemAddNodePhase::Idle);
        assert!(!retyped.view_state().can_import, "a url that was never checked cannot be imported");
    }

    #[test]
    fn test_a_failure_replaces_the_answer_and_blocks_the_import() {
        let failed = GemAddNodeSession::new(Chain::Ethereum)
            .on_input("https://node".to_string())
            .on_checked(check())
            .on_failed(GemAddNodeFailure::InvalidNetworkId);

        assert_eq!(
            failed.view_state().phase,
            GemAddNodePhase::Failed {
                failure: GemAddNodeFailure::InvalidNetworkId
            }
        );
        assert!(!failed.view_state().can_import);
    }

    #[test]
    fn test_importing_leaves_the_screen_ready_for_the_next_url() {
        let imported = GemAddNodeSession::new(Chain::Ethereum)
            .on_input("https://node".to_string())
            .on_checked(check())
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
}

#[cfg(test)]
mod node_list_tests {
    use super::*;
    use primitives::Latency;

    fn node(url: &str) -> GemNodeSelection {
        GemNodeSelection {
            url: url.to_string(),
            host: url.to_string(),
            is_selected: false,
            gem_node_flag: None,
        }
    }

    fn result(block: u64) -> GemNodeStatusState {
        GemNodeStatusState::Result {
            latest_block_number: block,
            latency: Latency::from_milliseconds(10),
        }
    }

    #[test]
    fn test_a_status_for_a_node_that_is_gone_is_dropped() {
        let session = GemNodeListSession::new(Chain::Ethereum)
            .on_nodes(vec![node("a"), node("b")])
            .on_status("a".to_string(), result(1))
            .on_status("b".to_string(), result(2));

        let after_delete = session.on_nodes(vec![node("a")]);

        assert_eq!(after_delete.statuses.len(), 1);
        assert!(after_delete.statuses.contains_key("a"));
    }

    #[test]
    fn test_a_status_that_arrives_for_a_node_the_list_no_longer_has_is_ignored() {
        let session = GemNodeListSession::new(Chain::Ethereum).on_nodes(vec![node("a")]);

        let late = session.on_status("gone".to_string(), result(3));

        assert!(late.statuses.is_empty(), "a late answer must not add a row back");
    }

    #[test]
    fn test_checking_puts_every_node_back_to_loading() {
        let session = GemNodeListSession::new(Chain::Ethereum)
            .on_nodes(vec![node("a"), node("b")])
            .on_status("a".to_string(), result(1))
            .on_checking();

        assert_eq!(session.node_urls(), vec!["a".to_string(), "b".to_string()]);
        assert!(session.statuses.values().all(|state| *state == GemNodeStatusState::Loading));
    }
}
