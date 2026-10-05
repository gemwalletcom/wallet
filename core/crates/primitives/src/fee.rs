use model_derive::Model;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

pub use crate::gas_price_type::GasPriceType;

pub const SOLANA_PRIORITY_FEE_SCALE: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq, PartialOrd, Ord, Model)]
#[model(swift = "Equatable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FeePriority {
    Normal,
    Fast,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq, Model)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[model(swift = "Equatable, Sendable")]
pub enum FeeUnitType {
    SatVb,
    Gwei,
    Native,
}

impl FeeUnitType {
    pub fn decimals(&self) -> u32 {
        match self {
            FeeUnitType::Native => 0,
            FeeUnitType::SatVb => 1,
            FeeUnitType::Gwei => 9,
        }
    }

    pub fn scale_factor(&self) -> u64 {
        10u64.pow(self.decimals())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeeRate {
    pub priority: FeePriority,
    pub gas_price_type: GasPriceType,
}

impl FeeRate {
    pub fn new(priority: FeePriority, gas_price_type: GasPriceType) -> Self {
        Self { priority, gas_price_type }
    }

    pub fn find(rates: &[FeeRate], priority: FeePriority) -> Option<&FeeRate> {
        rates.iter().find(|r| r.priority == priority)
    }
}

pub fn custom_fee_value(loaded_fee: &BigInt, loaded_price: &BigInt, price: &BigInt) -> BigInt {
    if loaded_price == &BigInt::ZERO { loaded_fee.clone() } else { loaded_fee * price / loaded_price }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_fee_value_scales_the_loaded_fee_by_the_price() {
        assert_eq!(custom_fee_value(&BigInt::from(1000), &BigInt::from(10), &BigInt::from(20)), BigInt::from(2000));
        assert_eq!(custom_fee_value(&BigInt::from(1000), &BigInt::ZERO, &BigInt::from(20)), BigInt::from(1000), "nothing scales against a zero price");
    }
}
