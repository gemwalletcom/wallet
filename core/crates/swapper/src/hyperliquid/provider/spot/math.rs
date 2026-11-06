use std::cmp::Ordering;

use num_bigint::BigUint;
use num_traits::Zero;
use number_formatter::{BigNumberFormatter, Decimal, Rounding, precision::round_to_places};

use crate::SwapperError;

pub(super) const SPOT_ASSET_OFFSET: u32 = 10_000;
const MAX_DECIMAL_SCALE: u32 = 6;

#[derive(Debug, Clone, Copy)]
pub(super) enum SpotSide {
    Buy,
    Sell,
}

impl SpotSide {
    pub(super) fn is_buy(self) -> bool {
        matches!(self, SpotSide::Buy)
    }
}

pub(super) fn format_decimal(value: &Decimal) -> String {
    BigNumberFormatter::decimal_to_string(value, MAX_DECIMAL_SCALE)
}

pub(super) fn round_size_down(amount: &Decimal, decimals: u32) -> Decimal {
    amount.with_scale_round(i64::from(decimals), Rounding::Down)
}

pub(super) fn format_order_size(amount: &Decimal, decimals: u32) -> String {
    let rounded = round_size_down(amount, decimals);
    BigNumberFormatter::decimal_to_string(&rounded, decimals)
}

pub(super) fn spot_asset_index(market_index: u32) -> u32 {
    SPOT_ASSET_OFFSET + market_index
}

pub fn scale_units(value: BigUint, from_decimals: u32, to_decimals: u32) -> Result<BigUint, SwapperError> {
    match from_decimals.cmp(&to_decimals) {
        Ordering::Equal => Ok(value),
        Ordering::Less => {
            let factor = BigUint::from(10u32).pow(to_decimals - from_decimals);
            Ok(value * factor)
        }
        Ordering::Greater => {
            let factor = BigUint::from(10u32).pow(from_decimals - to_decimals);
            if !(&value % &factor).is_zero() {
                return Err(SwapperError::ComputeQuoteError("amount precision loss".into()));
            }
            Ok(value / factor)
        }
    }
}

pub(super) fn limit_price_with_slippage(limit_price: &Decimal, side: SpotSide, slippage_bps: u32, price_decimals: u32) -> Result<Decimal, SwapperError> {
    if limit_price <= &Decimal::zero() {
        return Err(SwapperError::ComputeQuoteError("invalid limit price".into()));
    }

    let limit_price_f64 = limit_price.to_f64().ok_or_else(|| SwapperError::ComputeQuoteError("failed to convert price".into()))?;

    let slippage_fraction = slippage_bps as f64 / 10_000.0;
    let multiplier = if side.is_buy() { 1.0 + slippage_fraction } else { 1.0 - slippage_fraction };

    if multiplier <= 0.0 {
        return Err(SwapperError::ComputeQuoteError("slippage multiplier not positive".into()));
    }

    let adjusted = limit_price_f64 * multiplier;
    let rounded = round_to_significant_and_decimal(adjusted, 5, price_decimals);

    let formatted = if price_decimals == 0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.price_decimals$}", price_decimals = price_decimals as usize)
    };

    formatted.parse().map_err(|_| SwapperError::ComputeQuoteError("failed to format limit price".into()))
}

fn round_to_significant_and_decimal(value: f64, sig_figs: u32, max_decimals: u32) -> f64 {
    if value == 0.0 {
        return 0.0;
    }

    let abs_value = value.abs();
    let magnitude = abs_value.log10().floor() as i32;
    let scale = 10f64.powi(sig_figs as i32 - magnitude - 1);
    let rounded = (abs_value * scale).round() / scale;
    round_to_places(rounded.copysign(value), max_decimals as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decimal(value: &str) -> Decimal {
        value.parse().unwrap()
    }

    #[test]
    fn test_format_order_size() {
        assert_eq!(format_order_size(&decimal("0.131"), 2), "0.13");
        assert_eq!(format_order_size(&decimal("0.189834"), 2), "0.18");
        assert_eq!(format_order_size(&decimal("0.10"), 2), "0.1");
        assert_eq!(format_order_size(&decimal("9.999"), 2), "9.99");
    }

    #[test]
    fn test_round_size_down() {
        assert_eq!(round_size_down(&decimal("9.523768"), 2), decimal("9.52"));
        assert_eq!(round_size_down(&decimal("10.12345"), 2), decimal("10.12"));
        assert_eq!(round_size_down(&decimal("123.999"), 0), decimal("123"));
    }

    #[test]
    fn test_spot_asset_index() {
        assert_eq!(spot_asset_index(0), SPOT_ASSET_OFFSET);
        assert_eq!(spot_asset_index(107), SPOT_ASSET_OFFSET + 107);
    }

    #[test]
    fn test_limit_price_with_slippage() {
        let limit_price =
            |price: &str, side: SpotSide, slippage_bps: u32, price_decimals: u32| limit_price_with_slippage(&decimal(price), side, slippage_bps, price_decimals).map(|price| BigNumberFormatter::decimal_to_string(&price, price_decimals));
        assert_eq!(limit_price("100", SpotSide::Buy, 1000, 2), Ok("110".to_string()));
        assert_eq!(limit_price("100", SpotSide::Sell, 500, 2), Ok("95".to_string()));
        assert_eq!(limit_price("42.123456", SpotSide::Sell, 0, 4), Ok("42.123".to_string()));
        assert_eq!(limit_price("10", SpotSide::Sell, 10001, 2), Err(SwapperError::ComputeQuoteError("slippage multiplier not positive".into())));
        assert_eq!(limit_price("0", SpotSide::Buy, 100, 2), Err(SwapperError::ComputeQuoteError("invalid limit price".into())));
    }

    #[test]
    fn test_scale_units() {
        assert_eq!(scale_units(BigUint::from(123u32), 8, 18), Ok(BigUint::from(123u32) * BigUint::from(10u32).pow(10)));
        assert_eq!(scale_units(BigUint::from(123u32) * BigUint::from(10u32).pow(10), 18, 8), Ok(BigUint::from(123u32)));
        assert_eq!(scale_units(BigUint::from(5u32), 8, 8), Ok(BigUint::from(5u32)));
        assert_eq!(scale_units(BigUint::from(5u32), 3, 1), Err(SwapperError::ComputeQuoteError("amount precision loss".into())));
    }
}
