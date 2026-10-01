use std::time::Duration;

use crate::Chain;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JobConfiguration {
    pub initial_interval_ms: u32,
    pub max_interval_ms: u32,
    pub step_factor: f32,
}

impl Default for JobConfiguration {
    fn default() -> Self {
        Self {
            initial_interval_ms: 5_000,
            max_interval_ms: 60_000,
            step_factor: 1.1,
        }
    }
}

impl JobConfiguration {
    pub fn transaction_state(chain: Chain) -> Self {
        let config = Self::default();
        Self {
            initial_interval_ms: chain.block_time().clamp(1, config.initial_interval_ms),
            ..config
        }
    }

    pub fn next_interval_ms(&self, current_interval_ms: u32) -> u32 {
        let max_interval_ms = self.max_interval_ms.max(self.initial_interval_ms);
        let next_interval_ms = (f64::from(current_interval_ms) * f64::from(self.step_factor)) as u32;
        next_interval_ms.clamp(self.initial_interval_ms, max_interval_ms)
    }

    pub fn with_block_time(self, chain: Chain) -> Self {
        let max_interval_ms = self.max_interval_ms.max(self.initial_interval_ms);
        Self {
            initial_interval_ms: chain.block_time().clamp(self.initial_interval_ms, max_interval_ms),
            max_interval_ms: max_interval_ms.max(chain.block_time() / 2),
            ..self
        }
    }

    pub fn next_check_interval(&self, elapsed: Duration) -> Duration {
        let max_interval_ms = self.max_interval_ms.max(self.initial_interval_ms);
        let interval_ms = (elapsed.as_millis() as f64 * f64::from(self.step_factor - 1.0)) as u32;
        Duration::from_millis(u64::from(interval_ms.clamp(self.initial_interval_ms, max_interval_ms)))
    }

    pub fn max_interval_steps(&self) -> u32 {
        self.max_interval_ms.max(self.initial_interval_ms).div_ceil(self.initial_interval_ms.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_block_time() {
        let config = JobConfiguration {
            initial_interval_ms: 1_000,
            max_interval_ms: 30_000,
            step_factor: 2.5,
        };

        assert_eq!(config.with_block_time(Chain::Solana).initial_interval_ms, 1_000);
        assert_eq!(config.with_block_time(Chain::Ethereum).initial_interval_ms, Chain::Ethereum.block_time());
        assert_eq!(config.with_block_time(Chain::Bitcoin).initial_interval_ms, 30_000);
        assert_eq!(config.with_block_time(Chain::Bitcoin).max_interval_ms, 300_000);
        assert_eq!(config.with_block_time(Chain::Bitcoin).step_factor, 2.5);
        assert_eq!(config.with_block_time(Chain::Solana).max_interval_ms, 30_000);
    }

    #[test]
    fn test_next_check_interval() {
        let config = JobConfiguration {
            initial_interval_ms: 2_000,
            max_interval_ms: 30_000,
            step_factor: 2.5,
        };

        assert_eq!(config.next_check_interval(Duration::ZERO), Duration::from_millis(2_000));
        assert_eq!(config.next_check_interval(Duration::from_secs(2)), Duration::from_millis(3_000));
        assert_eq!(config.next_check_interval(Duration::from_secs(5)), Duration::from_millis(7_500));
        assert_eq!(config.next_check_interval(Duration::from_millis(12_500)), Duration::from_millis(18_750));
        assert_eq!(config.next_check_interval(Duration::from_secs(3_600)), Duration::from_millis(30_000));
    }

    #[test]
    fn test_transaction_state_configuration() {
        assert_eq!(
            JobConfiguration::default(),
            JobConfiguration {
                initial_interval_ms: 5_000,
                max_interval_ms: 60_000,
                step_factor: 1.1,
            }
        );

        let config = JobConfiguration::transaction_state(Chain::Ethereum);

        assert_eq!(config.initial_interval_ms, 5_000);
        assert_eq!(config.max_interval_ms, 60_000);
        assert_eq!(config.step_factor, 1.1);
    }

    #[test]
    fn test_next_interval_ms() {
        let mobile = JobConfiguration {
            initial_interval_ms: 5_000,
            max_interval_ms: 30_000,
            step_factor: 1.5,
        };
        assert_eq!(mobile.next_interval_ms(5_000), 7_500);
        assert_eq!(mobile.next_interval_ms(7_000), 10_500);
        assert_eq!(mobile.next_interval_ms(1_000), 5_000);
        assert_eq!(mobile.next_interval_ms(25_000), 30_000);

        let backend = JobConfiguration {
            initial_interval_ms: 60_000,
            max_interval_ms: 300_000,
            step_factor: 2.0,
        };
        assert_eq!(backend.next_interval_ms(60_000), 120_000);
        assert_eq!(backend.next_interval_ms(240_000), 300_000);
        assert_eq!(backend.next_interval_ms(300_000), 300_000);
        assert_eq!(backend.max_interval_steps(), 5);
    }
}
