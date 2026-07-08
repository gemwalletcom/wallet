use crate::rpc::proto::transactions::{ExecutionError, ExecutionStatus};
use crate::rpc::proto::{BalanceChange, TransactionEffects};

impl BalanceChange {
    pub fn mock(address: &str, coin_type: &str, amount: &str) -> Self {
        Self {
            address: Some(address.to_string()),
            coin_type: Some(coin_type.to_string()),
            amount: Some(amount.to_string()),
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
