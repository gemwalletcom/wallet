use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;
#[cfg(feature = "rpc")]
use serde_serializers::{deserialize_option_biguint_from_str, deserialize_u64_from_str};

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStakeDelegation {
    pub validator_address: String,
    pub staking_pool: String,
    pub stakes: Vec<SuiStake>,
}

#[derive(Debug, Clone)]
pub struct SuiSystemState {
    pub epoch: u64,
    pub epoch_start_ms: Option<i64>,
    pub epoch_duration_ms: Option<u64>,
}

#[cfg(feature = "rpc")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiStake {
    pub staked_sui_id: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub principal: BigUint,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub stake_active_epoch: u64,
    #[serde(default, deserialize_with = "deserialize_option_biguint_from_str")]
    pub estimated_reward: Option<BigUint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidators {
    pub apys: Vec<SuiValidator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiValidator {
    pub address: String,
    pub apy: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStake {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount: BigUint,
    pub staker_address: String,
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventUnstake {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub principal_amount: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub reward_amount: BigUint,
    pub staker_address: String,
    pub validator_address: String,
}
