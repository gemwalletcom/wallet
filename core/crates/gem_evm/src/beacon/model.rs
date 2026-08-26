use std::{error::Error, time::Duration};

use num_bigint::BigUint;
use serde::Deserialize;
use serde_serializers::deserialize_biguint_from_str;

const SECONDS_PER_EPOCH: u64 = 12 * 32;
const ACTIVATION_EXIT_CHURN_LIMIT_GWEI: u64 = 256_000_000_000;

#[derive(Debug, Deserialize)]
pub(crate) struct BeaconResponse<T> {
    execution_optimistic: bool,
    finalized: bool,
    data: T,
}

impl<T> BeaconResponse<T> {
    pub(crate) fn verified_data(self) -> Result<T, Box<dyn Error + Send + Sync>> {
        if !self.finalized || self.execution_optimistic {
            return Err("Beacon API returned an unverified state".into());
        }
        Ok(self.data)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct StateRoot {
    pub(crate) root: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PendingDeposit {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    amount: BigUint,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ValidatorEntry {
    validator: Validator,
}

#[derive(Debug, Deserialize)]
struct Validator {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    effective_balance: BigUint,
}

#[derive(Debug, PartialEq)]
pub struct ValidatorQueue {
    pub entry_queue_balance: BigUint,
    pub exit_queue_balance: BigUint,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidatorQueueEstimate {
    pub entry_wait_time: Duration,
    pub exit_wait_time: Duration,
}

impl ValidatorQueue {
    pub(crate) fn new(pending_deposits: Vec<PendingDeposit>, exiting_validators: Vec<ValidatorEntry>) -> Self {
        Self {
            entry_queue_balance: pending_deposits.into_iter().map(|deposit| deposit.amount).sum(),
            exit_queue_balance: exiting_validators.into_iter().map(|entry| entry.validator.effective_balance).sum(),
        }
    }

    pub fn estimated_wait_times(&self) -> Result<ValidatorQueueEstimate, Box<dyn Error + Send + Sync>> {
        Ok(ValidatorQueueEstimate {
            entry_wait_time: estimated_wait_time(&self.entry_queue_balance)?,
            exit_wait_time: estimated_wait_time(&self.exit_queue_balance)?,
        })
    }
}

fn estimated_wait_time(balance: &BigUint) -> Result<Duration, Box<dyn Error + Send + Sync>> {
    let seconds: u64 = (balance * SECONDS_PER_EPOCH / ACTIVATION_EXIT_CHURN_LIMIT_GWEI).try_into()?;
    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimated_wait_times() {
        let estimate = ValidatorQueue {
            entry_queue_balance: 2_197_019_000_000_000u64.into(),
            exit_queue_balance: 4_128_000_000_000u64.into(),
        }
        .estimated_wait_times()
        .unwrap();

        assert_eq!(
            estimate,
            ValidatorQueueEstimate {
                entry_wait_time: Duration::from_secs(3_295_528),
                exit_wait_time: Duration::from_secs(6_192),
            }
        );
        println!("Entry queue wait: {:?}", estimate.entry_wait_time);
        println!("Exit queue wait: {:?}", estimate.exit_wait_time);
    }
}
