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
    Reduce { available: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountRules {
    pub minimum_value: String,
    pub reserve_for_fee: String,
    pub can_change_value: bool,
    pub shows_asset_balance: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountLimits {
    pub available_value: String,
    pub max_value: String,
    pub reserves_fee: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemAmountError {
    InvalidValue { value: String },
    Zero,
    BelowMinimum { minimum: String },
    InsufficientBalance { available: String },
}

impl std::fmt::Display for GemAmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidValue { value } => write!(f, "invalid amount {value}"),
            Self::Zero => write!(f, "amount must be positive"),
            Self::BelowMinimum { minimum } => write!(f, "amount is below the minimum {minimum}"),
            Self::InsufficientBalance { available } => write!(f, "amount exceeds the available balance {available}"),
        }
    }
}

impl std::error::Error for GemAmountError {}
