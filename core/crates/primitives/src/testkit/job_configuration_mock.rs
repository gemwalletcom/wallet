use crate::JobConfiguration;

impl JobConfiguration {
    pub fn mock() -> Self {
        Self {
            initial_interval_ms: 1,
            max_interval_ms: 1,
            step_factor: 1.0,
        }
    }
}
