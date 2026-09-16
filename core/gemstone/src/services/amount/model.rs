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
pub enum GemAmountTitle {
    Send,
    Deposit,
    Withdraw,
    Stake,
    Unstake,
    Redelegate,
    Rewards,
    Freeze,
    Unfreeze,
    PerpetualOpen { direction: PerpetualDirection },
    PerpetualIncrease { direction: PerpetualDirection },
    PerpetualReduce { direction: PerpetualDirection },
}

#[uniffi::export]
impl GemAmountType {
    pub fn title(&self) -> GemAmountTitle {
        super::rules::amount_title(self)
    }
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

#[uniffi::export]
impl GemAmountInputType {
    pub fn toggled(&self) -> Self {
        match self {
            Self::Asset => Self::Fiat,
            Self::Fiat => Self::Asset,
        }
    }
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

#[uniffi::export]
impl GemAmountEntry {
    pub fn allows_confirm(&self) -> bool {
        self.error.is_none() && self.value.as_ref().is_some_and(|value| value.sign() == num_bigint::Sign::Plus)
    }
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAmountErrorDisplay {
    None,
    InvalidAmount,
    BelowMinimum { asset: Asset, minimum: GemBigInt },
    InsufficientBalance { title: String },
}

#[uniffi::export]
impl GemAmountError {
    pub fn display(&self) -> GemAmountErrorDisplay {
        match self {
            Self::Zero => GemAmountErrorDisplay::None,
            Self::InvalidNumber | Self::PriceMissing => GemAmountErrorDisplay::InvalidAmount,
            Self::BelowMinimum { asset, minimum } => GemAmountErrorDisplay::BelowMinimum {
                asset: asset.clone(),
                minimum: minimum.clone(),
            },
            Self::InsufficientBalance { asset, .. } => GemAmountErrorDisplay::InsufficientBalance { title: asset_title(asset) },
        }
    }
}

fn asset_title(asset: &Asset) -> String {
    match asset.name == asset.symbol {
        true => asset.name.clone(),
        false => asset.full_name(),
    }
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

    pub fn input_text(&self, value: String, decimals: u32) -> Option<String> {
        super::rules::input_text(&self.decimal_separator, &value, decimals)
    }

    pub fn plain(&self, input: String) -> String {
        super::rules::plain_number(&self.decimal_separator, &input)
    }

    pub fn value(&self, input: String, decimals: u32) -> Result<GemBigInt, GemAmountError> {
        super::rules::value_from_input(&self.decimal_separator, &input, decimals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_the_input_type_toggles_between_the_asset_and_the_fiat_side() {
        assert_eq!(GemAmountInputType::Asset.toggled(), GemAmountInputType::Fiat);
        assert_eq!(GemAmountInputType::Fiat.toggled(), GemAmountInputType::Asset);
        assert_eq!(GemAmountInputType::Asset.toggled().toggled(), GemAmountInputType::Asset);
    }

    #[test]
    fn test_a_zero_amount_shows_nothing_and_a_short_balance_names_the_asset() {
        assert_eq!(GemAmountError::Zero.display(), GemAmountErrorDisplay::None);
        assert_eq!(GemAmountError::InvalidNumber.display(), GemAmountErrorDisplay::InvalidAmount);
        assert_eq!(GemAmountError::PriceMissing.display(), GemAmountErrorDisplay::InvalidAmount);

        let asset = Asset::from_chain(Chain::Ethereum);
        let requirement = GemBalanceRequirement::new(GemBigInt::from(1), GemBigInt::ZERO);
        assert_eq!(
            GemAmountError::InsufficientBalance {
                asset: asset.clone(),
                requirement: requirement.clone(),
            }
            .display(),
            GemAmountErrorDisplay::InsufficientBalance {
                title: format!("{} ({})", asset.name, asset.symbol)
            }
        );

        let same = Asset {
            name: asset.symbol.clone(),
            ..asset
        };
        assert_eq!(
            GemAmountError::InsufficientBalance { asset: same.clone(), requirement }.display(),
            GemAmountErrorDisplay::InsufficientBalance { title: same.symbol }
        );
    }
}
