use primitives::{Chain, JobConfiguration, swap_transaction_timeout};

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

/// How long a transaction may stay unresolved before the clients give up on it.
///
/// `StatusProvider` already applies this when a status lookup succeeds; the
/// clients need it too, so a transaction whose status endpoint stays unreachable
/// does not poll forever. Pass the swap destination chain when the transaction is
/// in transit across chains, otherwise omit it.
#[uniffi::export]
pub fn transaction_timeout_ms(chain: Chain, destination_chain: Option<Chain>) -> u64 {
    swap_transaction_timeout(chain, destination_chain.unwrap_or(chain))
}
