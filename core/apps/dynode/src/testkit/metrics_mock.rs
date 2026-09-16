use crate::config::MetricsConfig;
use crate::metrics::Metrics;

impl Metrics {
    pub fn mock() -> Self {
        Self::new(MetricsConfig::mock())
    }
}
