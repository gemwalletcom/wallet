use num_bigint::BigUint;
#[cfg(feature = "rpc")]
use serde::Deserialize;
#[cfg(feature = "rpc")]
use serde_serializers::deserialize_f64_from_str;

use super::contracts::WithdrawRequest;

#[cfg(feature = "rpc")]
#[derive(Debug, Deserialize)]
pub struct StatsResponse {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub apr: f64,
}

#[derive(Debug)]
pub struct AccountState {
    pub deposited_balance: BigUint,
    pub pending_balance: BigUint,
    pub pending_deposited_balance: BigUint,
    pub withdraw_request: WithdrawRequest,
    pub restaked_reward: BigUint,
}
