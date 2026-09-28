use crate::perpetuals::PerpetualPositionClassifierConfig;

impl PerpetualPositionClassifierConfig {
    pub fn mock() -> Self {
        Self {
            trigger_bps: 100,
            liquidation_bps: 600,
            concurrency: 3,
        }
    }
}
