use std::time::Duration;

use super::observation::NodeStatusObservation;
use super::switch_reason::NodeSwitchReason;
use crate::config::Url;
use primitives::NodeStatusState;

#[derive(Debug)]
pub(super) struct NodeSwitchResult<'a> {
    pub(super) observation: &'a NodeStatusObservation,
    pub(super) reason: NodeSwitchReason,
}

pub(super) struct NodeSelectionPolicy;

impl NodeSelectionPolicy {
    pub(super) fn select_node<'a>(current: &Url, configured_observations: &'a [NodeStatusObservation], latency_threshold: Option<Duration>) -> Option<NodeSwitchResult<'a>> {
        let current_index = configured_observations.iter().position(|observation| observation.url == *current)?;
        let current_observation = &configured_observations[current_index];
        let current_usable = current_observation.is_usable(latency_threshold);
        let candidates = if current_usable {
            &configured_observations[..current_index]
        } else {
            configured_observations
        };
        let candidate = candidates
            .iter()
            .find(|observation| observation.url != *current && observation.is_usable(latency_threshold))?;
        let reason = switch_reason(current_observation, current_usable);

        Some(NodeSwitchResult { observation: candidate, reason })
    }
}

fn switch_reason(current_observation: &NodeStatusObservation, current_usable: bool) -> NodeSwitchReason {
    match &current_observation.state {
        NodeStatusState::Error { message } => NodeSwitchReason::CurrentNodeError {
            error: current_observation.monitor_error,
            message: message.clone(),
        },
        NodeStatusState::Healthy(status) if !status.in_sync => NodeSwitchReason::BlockHeight,
        NodeStatusState::Healthy(_) if !current_usable => NodeSwitchReason::Latency,
        NodeStatusState::Healthy(_) => NodeSwitchReason::PreferredNode,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::monitoring::switch_reason::NodeMonitorError;

    #[test]
    fn selects_highest_priority_healthy_node() {
        let configured = vec![
            NodeStatusObservation::mock_error("https://a", "unavailable"),
            NodeStatusObservation::mock_healthy("https://b", 110, 300),
            NodeStatusObservation::mock_healthy("https://c", 120, 20),
        ];
        let result = NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, None).unwrap();
        assert_eq!(result.observation.url, Url::mock("https://b"));

        let result = NodeSelectionPolicy::select_node(&Url::mock("https://c"), &configured, None).unwrap();
        assert_eq!(result.observation.url, Url::mock("https://b"));

        let configured = vec![
            NodeStatusObservation::mock_healthy("https://a", 100, 500),
            NodeStatusObservation::mock_healthy("https://b", 120, 10),
            NodeStatusObservation::mock_healthy("https://c", 120, 20),
        ];
        let result = NodeSelectionPolicy::select_node(&Url::mock("https://c"), &configured, None).unwrap();
        assert_eq!(result.observation.url, Url::mock("https://a"));
    }

    #[test]
    fn keeps_current_without_a_higher_priority_healthy_node() {
        let configured = vec![
            NodeStatusObservation::mock_healthy("https://a", 120, 100),
            NodeStatusObservation::mock_healthy("https://b", 120, 50),
        ];
        assert!(NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, None).is_none());

        let configured = vec![
            NodeStatusObservation::mock_error("https://a", "unavailable"),
            NodeStatusObservation::mock_healthy("https://b", 120, 100),
            NodeStatusObservation::mock_healthy("https://c", 120, 50),
        ];
        assert!(NodeSelectionPolicy::select_node(&Url::mock("https://b"), &configured, None).is_none());

        let configured = vec![
            NodeStatusObservation::mock_error("https://a", "unavailable"),
            NodeStatusObservation::mock_error("https://b", "unavailable"),
        ];
        assert!(NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, None).is_none());
        assert!(NodeSelectionPolicy::select_node(&Url::mock("https://missing"), &configured, None).is_none());
    }

    #[test]
    fn reports_switch_reason() {
        let configured = vec![
            NodeStatusObservation::mock_not_in_sync("https://a", 100, 90, 100),
            NodeStatusObservation::mock_healthy("https://b", 110, 500),
        ];
        let result = NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, None).unwrap();
        assert_eq!(result.reason, NodeSwitchReason::BlockHeight);

        let configured = vec![
            NodeStatusObservation::mock_error("https://a", "connection failed"),
            NodeStatusObservation::mock_healthy("https://b", 110, 500),
        ];
        let result = NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, None).unwrap();
        assert_eq!(
            result.reason,
            NodeSwitchReason::CurrentNodeError {
                error: NodeMonitorError::Request,
                message: "connection failed".to_string()
            }
        );

        let configured = vec![
            NodeStatusObservation::mock_healthy("https://a", 110, 500),
            NodeStatusObservation::mock_healthy("https://b", 110, 100),
        ];
        let result = NodeSelectionPolicy::select_node(&Url::mock("https://b"), &configured, None).unwrap();
        assert_eq!(result.reason, NodeSwitchReason::PreferredNode);
    }

    #[test]
    fn treats_slow_current_node_as_switch_candidate() {
        let configured = vec![
            NodeStatusObservation::mock_healthy("https://a", 120, 1200),
            NodeStatusObservation::mock_healthy("https://b", 120, 300),
        ];

        let result = NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, Some(Duration::from_secs(1))).unwrap();

        assert_eq!(result.observation.url, Url::mock("https://b"));
        assert_eq!(result.reason, NodeSwitchReason::Latency);
    }

    #[test]
    fn does_not_select_slow_candidate() {
        let configured = vec![
            NodeStatusObservation::mock_healthy("https://a", 120, 1200),
            NodeStatusObservation::mock_healthy("https://b", 120, 1100),
        ];

        assert!(NodeSelectionPolicy::select_node(&Url::mock("https://a"), &configured, Some(Duration::from_secs(1))).is_none());
    }
}
