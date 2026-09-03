use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{DELEGATION_POOL_GET_STAKE_FUNCTION, DELEGATION_POOL_OPERATOR_COMMISSION_FUNCTION, STAKE_GET_LOCKUP_SECS_FUNCTION};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOption<T> {
    pub vec: Vec<T>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewRequest {
    pub function: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<Value>,
}

impl ViewRequest {
    pub fn new(function: String, arguments: Vec<Value>) -> Self {
        Self {
            function,
            type_arguments: Vec::new(),
            arguments,
        }
    }

    pub fn delegation_pool_stake(pool: &str, delegator: &str) -> Self {
        Self::new(DELEGATION_POOL_GET_STAKE_FUNCTION.to_string(), vec![Value::from(pool), Value::from(delegator)])
    }

    pub fn operator_commission_percentage(pool: &str) -> Self {
        Self::new(DELEGATION_POOL_OPERATOR_COMMISSION_FUNCTION.to_string(), vec![Value::from(pool)])
    }

    pub fn stake_lockup_secs(pool: &str) -> Self {
        Self::new(STAKE_GET_LOCKUP_SECS_FUNCTION.to_string(), vec![Value::from(pool)])
    }
}
