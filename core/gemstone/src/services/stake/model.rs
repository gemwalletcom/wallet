use crate::models::custom_types::GemBigInt;
use crate::services::transfer::GemTransferData;
use primitives::Delegation;

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
pub struct GemClaimRewards {
    pub value: GemBigInt,
    pub destination: GemClaimRewardsDestination,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemClaimRewardsDestination {
    Transfer { transfer: GemTransferData },
    Amount { delegations: Vec<Delegation> },
}
