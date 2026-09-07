#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum StakeType {
    Stake(DelegationValidator),
    Unstake(Delegation),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
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
