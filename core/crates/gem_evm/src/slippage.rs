use alloy_primitives::U256;
use num_bigint::BigUint;
use primitives::swap::HUNDRED_PERCENT_IN_BPS;
use std::ops::{Add, Div, Mul, Rem};

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
    T: BasisPointConvert + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Add<Output = T>,
{
    bps_of(amount, HUNDRED_PERCENT_IN_BPS - bps.min(HUNDRED_PERCENT_IN_BPS))
}

pub fn bps_of<T>(amount: &T, bps: u32) -> T
where
    T: BasisPointConvert + Mul<Output = T> + Div<Output = T> + Rem<Output = T> + Add<Output = T>,
{
    let basis_points = T::from_u32(HUNDRED_PERCENT_IN_BPS);
    (amount.clone() / basis_points.clone()) * T::from_u32(bps) + (amount.clone() % basis_points.clone()) * T::from_u32(bps) / basis_points
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

    #[test]
    fn test_bps_of() {
        assert_eq!(bps_of(&U256::from(1_999), 50), U256::from(9));
        assert_eq!(bps_of(&1_999_u128, 50), 9_u128);
        assert_eq!(bps_of(&BigUint::from(10_000u64), 0), BigUint::ZERO);
        assert_eq!(bps_of(&10_000_u64, HUNDRED_PERCENT_IN_BPS), 10_000_u64);
        assert_eq!(bps_of(&u64::MAX, 50), u64::MAX / 200, "a u64 amount never overflows on the way");
        assert_eq!(subtract_bps(&u64::MAX, 100), (u128::from(u64::MAX) * 9_900 / 10_000) as u64);
    }
}
