#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(tag = "type", content = "content")]
pub enum StakeType {
    Stake(DelegationValidator),
    Unstake(Delegation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub validator: DelegationValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StakeAction {
    Delegate(DelegationValidator),
    Claim(Vec<DelegationValidator>),
    Reward(#[serde(serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_from_str")] Option<BigInt>),
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum StakeOwner {
    Wallet,
    Validator(DelegationValidator),
}
