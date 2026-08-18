use std::fmt;

use num_bigint::BigUint;

use crate::{Chain, asset::Asset};

const MAX_EXPONENT: u32 = 78;

pub(crate) fn decimal(value: &str) -> Option<String> {
    Some(Amount::parse(value)?.to_string())
}

pub(crate) fn exact(value: &str, chain: Chain) -> Option<String> {
    let decimals = u32::try_from(Asset::from_chain(chain).decimals).ok()?;
    let amount = Amount::parse(value)?;

    (amount.significant_decimals() <= decimals).then(|| amount.to_string())
}

pub(crate) fn exact_from_atomic(value: &str, chain: Chain) -> Option<String> {
    let decimals = u32::try_from(Asset::from_chain(chain).decimals).ok()?;
    let atomic = Amount::parse_atomic(value)?;

    Some(Amount { decimals, ..atomic }.to_string())
}

pub(crate) fn atomic(value: &str) -> Option<BigUint> {
    Amount::parse_atomic(value)?.units.parse().ok()
}

struct Amount {
    units: String,
    decimals: u32,
}

impl Amount {
    fn parse(value: &str) -> Option<Self> {
        let value = value.strip_prefix('+').unwrap_or(value);
        let (integer, fraction) = value.split_once('.').unwrap_or((value, ""));
        let units = format!("{integer}{fraction}");

        if units.is_empty() || !units.bytes().all(|digit| digit.is_ascii_digit()) {
            return None;
        }
        Some(Self {
            units,
            decimals: u32::try_from(fraction.len()).ok()?,
        })
    }

    fn parse_atomic(value: &str) -> Option<Self> {
        let (mantissa, exponent) = value.split_once(['e', 'E']).unwrap_or((value, "0"));
        let exponent: u32 = exponent.parse().ok().filter(|exponent| *exponent <= MAX_EXPONENT)?;
        let amount = Self::parse(mantissa)?;

        (amount.decimals <= exponent).then(|| Self {
            units: amount.units + &"0".repeat((exponent - amount.decimals) as usize),
            decimals: 0,
        })
    }

    fn significant_decimals(&self) -> u32 {
        let trailing_zeros = self.units.len() - self.units.trim_end_matches('0').len();

        self.decimals.saturating_sub(trailing_zeros as u32)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimals = self.decimals as usize;
        let units = format!("{:0>width$}", self.units, width = decimals + 1);
        let (integer, fraction) = units.split_at(units.len() - decimals);

        match fraction.trim_end_matches('0') {
            "" => f.write_str(integer),
            fraction => write!(f, "{integer}.{fraction}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal() {
        assert_eq!(decimal("50"), Some("50".to_string()));
        assert_eq!(decimal("0.500"), Some("0.5".to_string()));
        assert_eq!(decimal(".123"), Some("0.123".to_string()));

        assert_eq!(decimal("XYZ"), None);
        assert_eq!(decimal("100,000"), None);
        assert_eq!(decimal("1_000"), None);
        assert_eq!(decimal("-1"), None);
        assert_eq!(decimal(""), None);
        assert_eq!(decimal("1e2"), None);
    }

    #[test]
    fn test_exact() {
        assert_eq!(exact("0.0001", Chain::Bitcoin), Some("0.0001".to_string()));
        assert_eq!(exact("1.000000000", Chain::Bitcoin), Some("1".to_string()));

        assert_eq!(exact("0.000000001", Chain::Bitcoin), None);
        assert_eq!(exact("0.123456789", Chain::Bitcoin), None);
        assert_eq!(exact("0.000000001", Chain::Ethereum), Some("0.000000001".to_string()));
    }

    #[test]
    fn test_atomic() {
        assert_eq!(atomic("1500000"), Some(BigUint::from(1_500_000u32)));
        assert_eq!(atomic("1.5e6"), Some(BigUint::from(1_500_000u32)));
        assert_eq!(atomic("+2.014e18"), Some(BigUint::from(2_014_000_000_000_000_000u64)));

        assert_eq!(atomic("0.5"), None);
        assert_eq!(atomic("1.5"), None);
        assert_eq!(atomic("1e-6"), None);
        assert_eq!(atomic("XYZ"), None);
        assert_eq!(atomic("-1"), None);
        assert_eq!(atomic(""), None);
    }

    #[test]
    fn test_exact_from_atomic() {
        assert_eq!(exact_from_atomic("10000000", Chain::Bitcoin), Some("0.1".to_string()));
        assert_eq!(exact_from_atomic("1000000000", Chain::Ton), Some("1".to_string()));
        assert_eq!(exact_from_atomic("+2.014e18", Chain::Ethereum), Some("2.014".to_string()));

        assert_eq!(exact_from_atomic("0.5", Chain::Ton), None);
        assert_eq!(exact_from_atomic("123.3", Chain::Ton), None);

        assert_eq!(exact_from_atomic("1e", Chain::Ethereum), None);
        assert_eq!(exact_from_atomic("1e-6", Chain::Ethereum), None);
        assert_eq!(exact_from_atomic("1e79", Chain::Ethereum), None);
        assert_eq!(exact_from_atomic("1e4000000000", Chain::Ethereum), None);
    }
}
