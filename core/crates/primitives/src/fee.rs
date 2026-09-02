use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use typeshare::typeshare;

pub use crate::gas_price_type::GasPriceType;

pub const SOLANA_PRIORITY_FEE_SCALE: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
#[typeshare(swift = "Equatable, Sendable, CaseIterable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum FeePriority {
    Normal,
    Fast,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, AsRefStr, EnumString, EnumIter)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
#[typeshare(swift = "Equatable, Sendable")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomFee {
    pub fee_value: BigInt,
    pub max_rate: BigInt,
    pub minimum_rate: Option<BigInt>,
    pub is_over_max: bool,
    pub is_below_minimum: bool,
    pub is_valid: bool,
}

impl CustomFee {
    pub fn calculate(rate: Option<BigInt>, loaded_fee: BigInt, base_total: BigInt, normal_total: BigInt, max_multiplier: u32, minimum_rate: Option<BigInt>) -> Self {
        let rate = rate.filter(|value| value > &BigInt::from(0));
        let max_rate = normal_total * BigInt::from(max_multiplier);

        let (fee_value, is_over_max, is_below_minimum) = match &rate {
            Some(rate) => {
                let is_over_max = rate > &max_rate;
                let is_below_minimum = minimum_rate.as_ref().is_some_and(|minimum| rate < minimum);
                let fee_value = if base_total != BigInt::from(0) { &loaded_fee * rate / &base_total } else { loaded_fee };
                (fee_value, is_over_max, is_below_minimum)
            }
            None => (loaded_fee, false, false),
        };

        Self {
            fee_value,
            max_rate,
            minimum_rate,
            is_over_max,
            is_below_minimum,
            is_valid: rate.is_some() && !is_over_max && !is_below_minimum,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calculate(rate: Option<i64>, loaded_fee: i64, base_total: i64, normal_total: i64, max_multiplier: u32) -> CustomFee {
        calculate_with_minimum(rate, loaded_fee, base_total, normal_total, max_multiplier, None)
    }

    fn calculate_with_minimum(rate: Option<i64>, loaded_fee: i64, base_total: i64, normal_total: i64, max_multiplier: u32, minimum_rate: Option<i64>) -> CustomFee {
        CustomFee::calculate(
            rate.map(BigInt::from),
            BigInt::from(loaded_fee),
            BigInt::from(base_total),
            BigInt::from(normal_total),
            max_multiplier,
            minimum_rate.map(BigInt::from),
        )
    }

    #[test]
    fn test_calculate() {
        let fee = calculate(Some(20), 1000, 10, 10, 10);
        assert_eq!(fee.fee_value, BigInt::from(2000));
        assert_eq!(fee.max_rate, BigInt::from(100));
        assert!(!fee.is_over_max);
        assert!(fee.is_valid);

        let over = calculate(Some(101), 1000, 10, 10, 10);
        assert_eq!(over.fee_value, BigInt::from(10100));
        assert!(over.is_over_max);
        assert!(!over.is_valid);

        let empty = calculate(None, 1000, 10, 10, 10);
        assert_eq!(empty.fee_value, BigInt::from(1000));
        assert!(!empty.is_over_max);
        assert!(!empty.is_valid);

        let non_positive = calculate(Some(0), 1000, 10, 10, 10);
        assert_eq!(non_positive.fee_value, BigInt::from(1000));
        assert!(!non_positive.is_over_max);
        assert!(!non_positive.is_valid);

        let zero_base = calculate(Some(20), 1000, 0, 10, 10);
        assert_eq!(zero_base.fee_value, BigInt::from(1000));
        assert!(!zero_base.is_over_max);
    }

    #[test]
    fn test_calculate_rejects_a_rate_below_the_chain_minimum() {
        let below = calculate_with_minimum(Some(4), 1000, 10, 10, 10, Some(5));
        assert!(below.is_below_minimum);
        assert!(!below.is_valid);
        assert_eq!(below.minimum_rate, Some(BigInt::from(5)));

        let at_minimum = calculate_with_minimum(Some(5), 1000, 10, 10, 10, Some(5));
        assert!(!at_minimum.is_below_minimum);
        assert!(at_minimum.is_valid);

        let unbounded = calculate_with_minimum(Some(1), 1000, 10, 10, 10, None);
        assert!(!unbounded.is_below_minimum);
        assert!(unbounded.is_valid);

        let empty = calculate_with_minimum(None, 1000, 10, 10, 10, Some(5));
        assert!(!empty.is_below_minimum);
    }
}
