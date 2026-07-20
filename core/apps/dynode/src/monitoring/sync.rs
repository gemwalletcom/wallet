use std::time::Duration;

use super::switch_reason::{CurrentNodeErrorKind, NodeSwitchReason};
use crate::config::Url;
use primitives::{NodeStatusState, NodeSyncStatus};

#[derive(Debug, Clone)]
pub struct NodeStatusObservation {
    pub url: Url,
    pub state: NodeStatusState,
    pub latency: Duration,
    error_kind: CurrentNodeErrorKind,
}

impl NodeStatusObservation {
    pub fn new(url: Url, state: NodeStatusState, latency: Duration) -> Self {
        Self {
            url,
            state,
            latency,
            error_kind: CurrentNodeErrorKind::Unknown,
        }
    }

    pub(crate) fn with_error_kind(self, error_kind: CurrentNodeErrorKind) -> Self {
        Self { error_kind, ..self }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NodeSwitchResult {
    pub(crate) observation: NodeStatusObservation,
    pub(crate) reason: NodeSwitchReason,
}

pub struct NodeSyncAnalyzer;

impl NodeSyncAnalyzer {
    pub(crate) fn select_best_node(current: &Url, observations: &[NodeStatusObservation]) -> Option<NodeSwitchResult> {
        let current_observation = observations.iter().find(|o| o.url == *current)?;
        let (candidate, candidate_status) = Self::find_best_candidate(current, observations)?;
        let reason = match &current_observation.state {
            NodeStatusState::Error { message } => NodeSwitchReason::CurrentNodeError {
                kind: current_observation.error_kind.clone(),
                message: message.clone(),
            },
            NodeStatusState::Healthy(status) if !status.in_sync => NodeSwitchReason::BlockHeight {
                old_block: Self::status_height(status),
                new_block: Self::status_height(candidate_status),
            },
            NodeStatusState::Healthy(_) => return None,
        };

        Some(NodeSwitchResult {
            observation: candidate.clone(),
            reason,
        })
    }

    fn find_best_candidate<'a>(current: &Url, observations: &'a [NodeStatusObservation]) -> Option<(&'a NodeStatusObservation, &'a NodeSyncStatus)> {
        observations
            .iter()
            .filter(|observation| observation.url != *current)
            .filter_map(|observation| match observation.state.as_status() {
                Some(status) if status.in_sync => Some((observation, status)),
                _ => None,
            })
            .max_by(|(left_observation, left_status), (right_observation, right_status)| Self::compare_candidates(left_observation, left_status, right_observation, right_status))
    }

    pub fn format_status_summary(observations: &[NodeStatusObservation]) -> String {
        observations
            .iter()
            .map(|observation| match &observation.state {
                NodeStatusState::Healthy(status) => format!(
                    "{}:in_sync={} latest={} current={} latency={}ms",
                    observation.url.url,
                    status.in_sync,
                    Self::format_optional_number(status.latest_block_number),
                    Self::format_optional_number(status.current_block_number),
                    observation.latency.as_millis()
                ),
                NodeStatusState::Error { message } => format!("{}:error={} latency={}ms", observation.url.url, message, observation.latency.as_millis()),
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    fn format_optional_number(value: Option<u64>) -> String {
        value.map(|v| v.to_string()).unwrap_or_else(|| "unknown".to_string())
    }

    fn compare_candidates(
        left_observation: &NodeStatusObservation,
        left_status: &NodeSyncStatus,
        right_observation: &NodeStatusObservation,
        right_status: &NodeSyncStatus,
    ) -> std::cmp::Ordering {
        let left_height = Self::status_height(left_status);
        let right_height = Self::status_height(right_status);

        match left_height.cmp(&right_height) {
            std::cmp::Ordering::Equal => right_observation.latency.cmp(&left_observation.latency),
            other => other,
        }
    }

    fn status_height(status: &NodeSyncStatus) -> u64 {
        status.current_block_number.or(status.latest_block_number).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::sync::{healthy_observation, not_in_sync_observation, url};

    #[test]
    fn selects_highest_block_number() {
        let current = url("https://a");
        let observations = vec![
            not_in_sync_observation("https://a", Some(100), Some(90), 10),
            healthy_observation("https://b", Some(120), Some(120), 30),
            healthy_observation("https://c", Some(110), Some(110), 5),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
        assert_eq!(result.reason, NodeSwitchReason::BlockHeight { old_block: 90, new_block: 120 });
    }

    #[test]
    fn prioritizes_latency_on_equal_height() {
        let current = url("https://a");
        let observations = vec![
            not_in_sync_observation("https://a", Some(120), Some(110), 500),
            healthy_observation("https://b", Some(120), Some(120), 400),
            healthy_observation("https://c", Some(120), Some(120), 100),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://c");
    }

    #[test]
    fn ignores_unhealthy_nodes() {
        let current = url("https://a");
        let observations = vec![
            not_in_sync_observation("https://a", Some(100), Some(90), 10),
            healthy_observation("https://b", Some(120), Some(120), 40),
            NodeStatusObservation::new(url("https://c"), NodeStatusState::error("rpc error"), Duration::from_millis(5)),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
    }

    #[test]
    fn reports_none_when_no_candidate() {
        let current = url("https://a");
        let observations = vec![
            not_in_sync_observation("https://a", Some(100), Some(90), 10),
            NodeStatusObservation::new(url("https://b"), NodeStatusState::error("rpc"), Duration::from_millis(5)),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }

    #[test]
    fn switches_when_current_node_has_error() {
        let current = url("https://a");
        let observations = vec![
            NodeStatusObservation::new(url("https://a"), NodeStatusState::error("connection failed"), Duration::from_millis(10)),
            healthy_observation("https://b", Some(120), Some(120), 40),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
        assert_eq!(
            result.reason,
            NodeSwitchReason::CurrentNodeError {
                kind: CurrentNodeErrorKind::Unknown,
                message: "connection failed".to_string()
            }
        );
    }

    #[test]
    fn returns_none_when_current_node_not_found() {
        let current = url("https://a");
        let observations = vec![healthy_observation("https://b", Some(120), Some(120), 40)];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }

    #[test]
    fn returns_none_when_current_has_error_and_no_healthy_candidates() {
        let current = url("https://a");
        let observations = vec![
            NodeStatusObservation::new(url("https://a"), NodeStatusState::error("connection failed"), Duration::from_millis(10)),
            NodeStatusObservation::new(url("https://b"), NodeStatusState::error("also failed"), Duration::from_millis(20)),
        ];

        assert!(NodeSyncAnalyzer::select_best_node(&current, &observations).is_none());
    }

    #[test]
    fn not_in_sync_switches_to_synced_even_if_slower() {
        let current = url("https://a");
        let observations = vec![
            not_in_sync_observation("https://a", Some(100), Some(90), 100),
            healthy_observation("https://b", Some(100), Some(100), 500),
        ];

        let result = NodeSyncAnalyzer::select_best_node(&current, &observations).unwrap();
        assert_eq!(result.observation.url.url, "https://b");
    }
}
