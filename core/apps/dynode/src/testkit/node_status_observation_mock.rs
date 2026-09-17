use std::time::Duration;

use primitives::{NodeStatusState, NodeSyncStatus};

use crate::config::Url;
use crate::monitoring::observation::NodeStatusObservation;

impl NodeStatusObservation {
    pub fn mock_healthy(host: &str, block: u64, latency_ms: u64) -> Self {
        let status = NodeSyncStatus::new(true, Some(block), Some(block));
        Self::new(Url::mock(host), NodeStatusState::healthy(status), Duration::from_millis(latency_ms))
    }

    pub fn mock_not_in_sync(host: &str, latest: u64, current: u64, latency_ms: u64) -> Self {
        let status = NodeSyncStatus::new(false, Some(latest), Some(current));
        Self::new(Url::mock(host), NodeStatusState::healthy(status), Duration::from_millis(latency_ms))
    }

    pub fn mock_error(host: &str, message: &str) -> Self {
        Self::new(Url::mock(host), NodeStatusState::error(message), Duration::from_millis(10))
    }
}
