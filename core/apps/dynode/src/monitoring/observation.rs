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
