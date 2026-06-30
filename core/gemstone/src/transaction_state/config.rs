use primitives::{Chain, JobConfiguration};

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemJobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}

impl From<JobConfiguration> for GemJobConfiguration {
    fn from(config: JobConfiguration) -> Self {
        Self {
            initial_interval_ms: config.initial_interval_ms,
            max_interval_ms: config.max_interval_ms,
            step_factor: config.step_factor,
        }
    }
}

impl From<GemJobConfiguration> for JobConfiguration {
    fn from(config: GemJobConfiguration) -> Self {
        Self {
            initial_interval_ms: config.initial_interval_ms,
            max_interval_ms: config.max_interval_ms,
            step_factor: config.step_factor,
        }
    }
}

#[uniffi::export]
impl GemJobConfiguration {
    pub fn next_interval_ms(&self, current_interval_ms: u32) -> u32 {
        JobConfiguration::from(*self).next_interval_ms(current_interval_ms)
    }
}

#[uniffi::export]
pub fn transaction_state_config(chain: Chain) -> GemJobConfiguration {
    JobConfiguration::transaction_state(chain).into()
}
