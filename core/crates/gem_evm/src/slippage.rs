use alloy_primitives::U256;
use num_bigint::BigUint;
use primitives::swap::HUNDRED_PERCENT_IN_BPS;
use std::ops::{Div, Mul};

pub trait BasisPointConvert: Sized + Clone {
    fn from_u32(value: u32) -> Self;
}

impl BasisPointConvert for U256 {
    fn from_u32(value: u32) -> Self {
        Self::from(value)
    }
}

impl BasisPointConvert for u128 {
    fn from_u32(value: u32) -> Self {
        value as u128
    }
}

impl BasisPointConvert for u64 {
    fn from_u32(value: u32) -> Self {
        value as u64
    }
}

impl BasisPointConvert for BigUint {
    fn from_u32(value: u32) -> Self {
        Self::from(value)
    }
}

pub fn subtract_bps<T>(amount: &T, bps: u32) -> T
where
    T: BasisPointConvert + Mul<Output = T> + Div<Output = T>,
{
    let basis_points = T::from_u32(HUNDRED_PERCENT_IN_BPS);
    let remaining = T::from_u32(HUNDRED_PERCENT_IN_BPS - bps.min(HUNDRED_PERCENT_IN_BPS));
    (amount.clone() * remaining) / basis_points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtract_bps() {
        assert_eq!(subtract_bps(&U256::from(100), 300), U256::from(97));
        assert_eq!(subtract_bps(&100_u128, 300), 97_u128);
        assert_eq!(subtract_bps(&1000_u64, 500), 950_u64);
        assert_eq!(subtract_bps(&U256::from(1000), 0), U256::from(1000));
        assert_eq!(subtract_bps(&U256::from(1000), HUNDRED_PERCENT_IN_BPS), U256::ZERO);
        assert_eq!(subtract_bps(&BigUint::from(2_132_525u64), 100), BigUint::from(2_111_199u64));
    }
}
