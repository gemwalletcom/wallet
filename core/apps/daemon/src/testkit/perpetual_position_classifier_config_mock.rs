use crate::worker::perpetuals::perpetual_classifier::PerpetualPositionClassifierConfig;

impl PerpetualPositionClassifierConfig {
    pub fn mock() -> Self {
        Self {
            trigger_bps: 100,
            liquidation_bps: 600,
            concurrency: 3,
        }
    }
}
