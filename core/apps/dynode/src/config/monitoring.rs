use std::time::Duration;

use primitives::NodeCheckProfile;
use serde::Deserialize;
use serde_serializers::duration;

#[derive(Debug, Deserialize, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    #[serde(default)]
    pub profile: NodeCheckProfile,
    #[serde(deserialize_with = "duration::deserialize")]
    pub interval: Duration,
    pub trigger: FailureTriggerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FailureTriggerConfig {
    pub failures: usize,
    pub rate: u8,
    #[serde(deserialize_with = "duration::deserialize")]
    pub window: Duration,
    #[serde(default, deserialize_with = "duration::deserialize_option")]
    pub latency: Option<Duration>,
}
