use crate::models::custom_types::GemBigInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemDelegationAction {
    Stake,
    Unstake,
    Redelegate,
    Withdraw,
    Deposit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemStakeAction {
    Stake,
    Freeze,
    Unfreeze,
    ClaimRewards,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemStakeActionItem {
    pub action: GemStakeAction,
    pub is_enabled: bool,
    pub requires_frozen_balance: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemStakeBalance {
    pub frozen: GemBigInt,
    pub locked: GemBigInt,
    pub staked: GemBigInt,
    pub pending: GemBigInt,
    pub rewards: GemBigInt,
}
