use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::services::balance::GemBalanceRequirement;
use primitives::{Delegation, Resource};

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemAmountType {
    Transfer,
    Deposit,
    Withdraw,
    Stake {
        stake_type: GemAmountStakeType,
    },
    Earn {
        earn_type: GemAmountEarnType,
    },
    Perpetual {
        position: GemAmountPerpetualPosition,
        price: f64,
        leverage: u8,
        size_decimals: i32,
    },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemAmountStakeType {
    Stake,
    Unstake { delegation: Delegation },
    Redelegate { delegation: Delegation },
    Withdraw { delegation: Delegation },
    Rewards { delegations: Vec<Delegation> },
    Freeze { resource: Resource },
    Unfreeze { resource: Resource },
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemAmountEarnType {
    Deposit,
    Withdraw { delegation: Delegation },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemAmountPerpetualPosition {
    Open,
    Increase,
    Reduce { available: GemBigUint },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountInput {
    pub available_value: GemBigInt,
    pub max_value: GemBigInt,
    pub reserved_fee: Option<GemBigInt>,
    pub can_change_value: bool,
    pub shows_asset_balance: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualAutoclose {
    pub take_profit: Option<f64>,
    pub stop_loss: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemAmountError {
    Zero,
    BelowMinimum { minimum: GemBigInt },
    InsufficientBalance { requirement: GemBalanceRequirement },
}

impl std::fmt::Display for GemAmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => write!(f, "amount must be positive"),
            Self::BelowMinimum { minimum } => write!(f, "amount is below the minimum {minimum}"),
            Self::InsufficientBalance { requirement } => write!(f, "amount exceeds the available balance {}", requirement.available),
        }
    }
}

impl std::error::Error for GemAmountError {}
