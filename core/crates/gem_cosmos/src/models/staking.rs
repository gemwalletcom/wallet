use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, deserialize_f64_from_str};

use super::account::{Balance, Page, Pagination};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegations {
    pub delegation_responses: Vec<Delegation>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

impl Page for Delegations {
    type Item = Delegation;

    fn into_items(self) -> Vec<Delegation> {
        self.delegation_responses
    }

    fn next_page_key(&self) -> Option<String> {
        Pagination::key(&self.pagination)
    }
}

impl Page for UnbondingDelegations {
    type Item = UnbondingDelegation;

    fn into_items(self) -> Vec<UnbondingDelegation> {
        self.unbonding_responses
    }

    fn next_page_key(&self) -> Option<String> {
        Pagination::key(&self.pagination)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegation: DelegationData,
    pub balance: Balance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationData {
    pub validator_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingDelegations {
    pub unbonding_responses: Vec<UnbondingDelegation>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingDelegation {
    pub validator_address: String,
    pub entries: Vec<UnbondingDelegationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnbondingDelegationEntry {
    pub completion_time: String,
    pub creation_height: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub balance: BigUint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rewards {
    pub rewards: Vec<Reward>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reward {
    pub validator_address: String,
    pub reward: Vec<Balance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorsResponse {
    pub validators: Vec<Validator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub operator_address: String,
    pub jailed: bool,
    pub status: String,
    pub description: ValidatorDescription,
    pub commission: ValidatorCommission,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDescription {
    pub moniker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommission {
    pub commission_rates: ValidatorCommissionRates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorCommissionRates {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPoolResponse {
    pub pool: StakingPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub bonded_tokens: f64,
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub not_bonded_tokens: f64,
}
