use num_bigint::BigInt;

use crate::rpc::proto::service::{StakingPool, SystemState, Validator, ValidatorSet};
use crate::rpc::proto::transactions::{ExecutionError, ExecutionStatus};
use crate::rpc::proto::{BalanceChange, Epoch, TransactionEffects};

pub(crate) const TEST_VALIDATOR_ADDRESS: &str = "validator";
const TEST_POOL_ID: &str = "pool";
const TEST_POOL_SCALE: u64 = 1_000_000_000_000_000;

impl BalanceChange {
    pub fn mock(address: &str, coin_type: &str, amount: i64) -> Self {
        Self {
            address: Some(address.to_string()),
            coin_type: Some(coin_type.to_string()),
            amount: Some(BigInt::from(amount)),
        }
    }
}

impl TransactionEffects {
    pub fn mock(success: bool, error: Option<&str>) -> Self {
        Self {
            status: Some(ExecutionStatus {
                success: Some(success),
                error: error.map(|description| ExecutionError {
                    description: Some(description.to_string()),
                }),
            }),
            ..Default::default()
        }
    }
}

impl Epoch {
    pub fn mock_with_validator_rate(epoch: u64, rate: f64) -> Self {
        Self {
            epoch,
            system_state: Some(SystemState {
                validators: Some(ValidatorSet {
                    active_validators: vec![Validator {
                        address: Some(TEST_VALIDATOR_ADDRESS.to_string()),
                        staking_pool: Some(StakingPool {
                            id: Some(TEST_POOL_ID.to_string()),
                            sui_balance: Some(TEST_POOL_SCALE),
                            pool_token_balance: Some((TEST_POOL_SCALE as f64 * rate).round() as u64),
                        }),
                    }],
                }),
                parameters: None,
            }),
            ..Default::default()
        }
    }
}
