use num_bigint::BigUint;

pub struct BscValidator {
    pub operator_address: String,
    pub moniker: String,
    pub commission: u64,
    pub apy: u64,
    pub jailed: bool,
}

pub struct BscDelegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: BigUint,
    pub shares: BigUint,
}

pub struct BscUndelegation {
    pub delegator_address: String,
    pub validator_address: String,
    pub amount: BigUint,
    pub shares: BigUint,
    pub unlock_time: Option<u64>,
}
