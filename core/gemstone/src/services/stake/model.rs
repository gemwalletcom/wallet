use crate::models::custom_types::GemBigInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDelegationAction {
    Stake,
    Unstake,
    Redelegate,
    Withdraw,
    Deposit,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeBalance {
    pub frozen: GemBigInt,
    pub locked: GemBigInt,
    pub staked: GemBigInt,
    pub pending: GemBigInt,
    pub rewards: GemBigInt,
}
