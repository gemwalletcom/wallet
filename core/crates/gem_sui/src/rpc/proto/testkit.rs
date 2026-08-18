use num_bigint::BigInt;

use crate::rpc::proto::transactions::{ExecutionError, ExecutionStatus};
use crate::rpc::proto::{BalanceChange, TransactionEffects};

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
