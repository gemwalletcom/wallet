use super::observation::NodeStatusObservation;
use super::switch_reason::NodeSwitchReason;
use crate::config::Url;
use primitives::{NodeStatusState, NodeSyncStatus};

#[derive(Debug)]
pub(super) struct NodeSwitchResult<'a> {
    pub(super) observation: &'a NodeStatusObservation,
    pub(super) reason: NodeSwitchReason,
}

pub(super) struct NodeSelectionPolicy;

impl NodeSelectionPolicy {
    pub(super) fn select_node<'a>(current: &Url, configured_observations: &'a [NodeStatusObservation]) -> Option<NodeSwitchResult<'a>> {
        let current_index = configured_observations.iter().position(|observation| observation.url == *current)?;
        let current_observation = &configured_observations[current_index];
        let candidates = if current_observation.state.is_healthy() {
            &configured_observations[..current_index]
        } else {
            configured_observations
        };
        let candidate = candidates.iter().find(|observation| observation.url != *current && observation.state.is_healthy())?;
        let candidate_status = candidate.state.as_status()?;
        let reason = match &current_observation.state {
            NodeStatusState::Error { message } => NodeSwitchReason::CurrentNodeError {
                kind: current_observation.error_kind.clone(),
                message: message.clone(),
            },
            NodeStatusState::Healthy(status) if !status.in_sync => NodeSwitchReason::BlockHeight {
                old_block: Self::status_height(status),
                new_block: Self::status_height(candidate_status),
            },
            NodeStatusState::Healthy(_) => NodeSwitchReason::PreferredNode,
        };

        Some(NodeSwitchResult { observation: candidate, reason })
    }

    fn status_height(status: &NodeSyncStatus) -> u64 {
        status.current_block_number.or(status.latest_block_number).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::monitoring::switch_reason::CurrentNodeErrorKind;
    use crate::testkit::sync::{healthy_observation, not_in_sync_observation, url};

    fn error_observation(host: &str, message: &str) -> NodeStatusObservation {
        NodeStatusObservation::new(url(host), NodeStatusState::error(message), Duration::from_millis(10))
    }

    #[test]
    fn selects_highest_priority_healthy_node() {
        let configured = vec![
            error_observation("https://a", "unavailable"),
            healthy_observation("https://b", Some(110), Some(110), 300),
            healthy_observation("https://c", Some(120), Some(120), 20),
        ];
        let result = NodeSelectionPolicy::select_node(&url("https://a"), &configured).unwrap();
        assert_eq!(result.observation.url, url("https://b"));

        let result = NodeSelectionPolicy::select_node(&url("https://c"), &configured).unwrap();
        assert_eq!(result.observation.url, url("https://b"));

        let configured = vec![
            healthy_observation("https://a", Some(100), Some(100), 500),
            healthy_observation("https://b", Some(120), Some(120), 10),
            healthy_observation("https://c", Some(120), Some(120), 20),
        ];
        let result = NodeSelectionPolicy::select_node(&url("https://c"), &configured).unwrap();
        assert_eq!(result.observation.url, url("https://a"));
    }

    #[test]
    fn keeps_current_without_a_higher_priority_healthy_node() {
        let configured = vec![
            healthy_observation("https://a", Some(120), Some(120), 100),
            healthy_observation("https://b", Some(120), Some(120), 50),
        ];
        assert!(NodeSelectionPolicy::select_node(&url("https://a"), &configured).is_none());

        let configured = vec![
            error_observation("https://a", "unavailable"),
            healthy_observation("https://b", Some(120), Some(120), 100),
            healthy_observation("https://c", Some(120), Some(120), 50),
        ];
        assert!(NodeSelectionPolicy::select_node(&url("https://b"), &configured).is_none());

        let configured = vec![error_observation("https://a", "unavailable"), error_observation("https://b", "unavailable")];
        assert!(NodeSelectionPolicy::select_node(&url("https://a"), &configured).is_none());
        assert!(NodeSelectionPolicy::select_node(&url("https://missing"), &configured).is_none());
    }

    #[test]
    fn reports_switch_reason() {
        let configured = vec![
            not_in_sync_observation("https://a", Some(100), Some(90), 100),
            healthy_observation("https://b", Some(110), Some(110), 500),
        ];
        let result = NodeSelectionPolicy::select_node(&url("https://a"), &configured).unwrap();
        assert_eq!(result.reason, NodeSwitchReason::BlockHeight { old_block: 90, new_block: 110 });

        let configured = vec![
            error_observation("https://a", "connection failed"),
            healthy_observation("https://b", Some(110), Some(110), 500),
        ];
        let result = NodeSelectionPolicy::select_node(&url("https://a"), &configured).unwrap();
        assert_eq!(
            result.reason,
            NodeSwitchReason::CurrentNodeError {
                kind: CurrentNodeErrorKind::Unknown,
                message: "connection failed".to_string()
            }
        );

        let configured = vec![
            healthy_observation("https://a", Some(110), Some(110), 500),
            healthy_observation("https://b", Some(110), Some(110), 100),
        ];
        let result = NodeSelectionPolicy::select_node(&url("https://b"), &configured).unwrap();
        assert_eq!(result.reason, NodeSwitchReason::PreferredNode);
    }
}
