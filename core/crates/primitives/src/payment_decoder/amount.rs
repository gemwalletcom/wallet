use std::fmt;

use crate::{Chain, asset::Asset};

const MAX_EXPONENT: u32 = 78;

pub(crate) fn from_coins(value: &str) -> Option<String> {
    Some(Amount::read(value)?.to_string())
}

pub(crate) fn from_smallest_unit(value: &str, chain: Chain) -> Option<String> {
    let decimals = u32::try_from(Asset::from_chain(chain).decimals).ok()?;

    Some(Amount::read_exponential(value)?.with_decimals(decimals)?.to_string())
}

struct Amount {
    units: String,
    decimals: u32,
}

impl Amount {
    fn read(value: &str) -> Option<Self> {
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

    fn read_exponential(value: &str) -> Option<Self> {
        let (mantissa, exponent) = value.split_once(['e', 'E']).unwrap_or((value, "0"));
        let exponent: u32 = exponent.parse().ok().filter(|exponent| *exponent <= MAX_EXPONENT)?;
        let amount = Self::read(mantissa)?;

        Some(Self {
            units: amount.units + &"0".repeat(exponent.saturating_sub(amount.decimals) as usize),
            decimals: amount.decimals.saturating_sub(exponent),
        })
    }

    fn with_decimals(self, decimals: u32) -> Option<Self> {
        (self.decimals == 0).then_some(Self { decimals, ..self })
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
    fn test_from_coins() {
        assert_eq!(from_coins("50"), Some("50".to_string()));
        assert_eq!(from_coins("0.500"), Some("0.5".to_string()));
        assert_eq!(from_coins(".123"), Some("0.123".to_string()));

        for not_a_plain_decimal in ["XYZ", "100,000", "1_000", "-1", "", "1e2"] {
            assert_eq!(from_coins(not_a_plain_decimal), None, "{not_a_plain_decimal}");
        }
    }

    #[test]
    fn test_from_smallest_unit() {
        assert_eq!(from_smallest_unit("10000000", Chain::Bitcoin), Some("0.1".to_string()));
        assert_eq!(from_smallest_unit("1000000000", Chain::Ton), Some("1".to_string()));
        assert_eq!(from_smallest_unit("+2.014e18", Chain::Ethereum), Some("2.014".to_string()));

        for fraction_of_a_unit in ["0.5", "123.3"] {
            assert_eq!(from_smallest_unit(fraction_of_a_unit, Chain::Ton), None, "{fraction_of_a_unit}");
        }
        for unusable_exponent in ["1e", "1e-6", "1e79", "1e4000000000"] {
            assert_eq!(from_smallest_unit(unusable_exponent, Chain::Ethereum), None, "{unusable_exponent}");
        }
    }
}
