use std::time::Duration;

use primitives::NodeStatusState;

use super::switch_reason::NodeMonitorError;
use crate::config::Url;

#[derive(Debug)]
pub(crate) struct NodeStatusObservation {
    pub(crate) url: Url,
    pub(crate) state: NodeStatusState,
    pub(crate) latency: Duration,
    pub(super) monitor_error: NodeMonitorError,
}

impl NodeStatusObservation {
    pub(crate) fn new(url: Url, state: NodeStatusState, latency: Duration) -> Self {
        Self {
            url,
            state,
            latency,
            monitor_error: NodeMonitorError::Request,
        }
    }

    pub(super) fn with_monitor_error(self, monitor_error: NodeMonitorError) -> Self {
        Self { monitor_error, ..self }
    }

    pub(super) fn is_usable(&self, latency_threshold: Option<Duration>) -> bool {
        self.state.is_healthy() && latency_threshold.is_none_or(|threshold| self.latency <= threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::config::url;
    use crate::testkit::sync::{healthy_observation, not_in_sync_observation};

    #[test]
    fn usability_respects_health_and_latency_threshold() {
        let cases = [
            (healthy_observation("https://a", Some(1), Some(1), 100), None, true),
            (healthy_observation("https://a", Some(1), Some(1), 100), Some(Duration::from_millis(100)), true),
            (healthy_observation("https://a", Some(1), Some(1), 101), Some(Duration::from_millis(100)), false),
            (not_in_sync_observation("https://a", Some(2), Some(1), 10), Some(Duration::from_millis(100)), false),
            (
                NodeStatusObservation::new(url("https://a"), NodeStatusState::error("unavailable"), Duration::from_millis(10)),
                Some(Duration::from_millis(100)),
                false,
            ),
        ];

        for (observation, threshold, expected) in cases {
            assert_eq!(observation.is_usable(threshold), expected);
        }
    }
}
