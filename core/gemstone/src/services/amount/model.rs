use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::payment::GemPaymentRecipient;
use crate::services::balance::GemBalanceRequirement;
use primitives::{Asset, Delegation, PerpetualDirection, Resource};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
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
        direction: PerpetualDirection,
        price: f64,
        leverage: u8,
        size_decimals: i32,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountTransfer {
    Send { payment: GemPaymentRecipient },
    Deposit,
    Withdraw,
}

#[uniffi::export]
impl GemAmountTransfer {
    pub fn amount_type(&self) -> GemAmountType {
        super::rules::transfer_amount_type(self)
    }

    pub fn display_asset(&self, asset: Asset) -> Asset {
        super::rules::transfer_display_asset(self, asset)
    }

    pub fn prefilled_amount(&self) -> Option<String> {
        super::rules::transfer_prefilled_amount(self)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountStakeType {
    Stake,
    Unstake { delegation: Delegation },
    Redelegate { delegation: Delegation },
    Withdraw { delegation: Delegation },
    Rewards { delegations: Vec<Delegation> },
    Freeze { resource: Resource },
    Unfreeze { resource: Resource },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemAmountEarnType {
    Deposit,
    Withdraw { delegation: Delegation },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
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
    pub uses_whole_amounts: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemAmountInputType {
    Asset,
    Fiat,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountEquivalent {
    Fiat { amount: f64 },
    Asset { value: GemBigInt },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountEntry {
    pub value: Option<GemBigInt>,
    pub error: Option<GemAmountError>,
    pub equivalent: Option<GemAmountEquivalent>,
    pub is_max: bool,
    pub reserved_fee: Option<GemBigInt>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAmountMaxEntry {
    pub input_type: GemAmountInputType,
    pub value: GemBigInt,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualAutoclose {
    pub take_profit: Option<f64>,
    pub stop_loss: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemAmountError {
    InvalidNumber,
    PriceMissing,
    Zero,
    BelowMinimum { asset: Asset, minimum: GemBigInt },
    InsufficientBalance { asset: Asset, requirement: GemBalanceRequirement },
}

impl std::fmt::Display for GemAmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumber => write!(f, "amount is not a number"),
            Self::PriceMissing => write!(f, "amount needs a price to convert from fiat"),
            Self::Zero => write!(f, "amount must be positive"),
            Self::BelowMinimum { asset, minimum } => write!(f, "amount is below the minimum {minimum} {}", asset.symbol),
            Self::InsufficientBalance { asset, requirement } => write!(f, "amount exceeds the available {} balance {}", asset.symbol, requirement.available),
        }
    }
}

impl std::error::Error for GemAmountError {}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNumberFormat {
    pub decimal_separator: String,
}

#[uniffi::export]
impl GemNumberFormat {
    pub fn sanitize(&self, input: String, maximum_fraction_digits: Option<u32>, maximum_integer_digits: Option<u32>) -> String {
        super::rules::sanitize_number_input(&self.decimal_separator, &input, maximum_fraction_digits, maximum_integer_digits)
    }

    pub fn plain(&self, input: String) -> String {
        super::rules::plain_number(&self.decimal_separator, &input)
    }
}
