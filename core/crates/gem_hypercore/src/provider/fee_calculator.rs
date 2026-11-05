use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, asset_constants::HYPERCORE_SPOT_USDC_ASSET_ID, swap::SwapData};
use std::error::Error;

use crate::perpetual_formatter::USDC_DECIMALS_MULTIPLIER;

pub const BUILDER_FEE_RATE_SCALE: u32 = 100_000;

pub fn calculate_perpetual_fee_amount(fiat_value: f64, fee_rate: f64, builder_fee_bps: u32) -> BigInt {
    fee_amount_in_usdc(fiat_value, fee_rate + builder_fee_rate(builder_fee_bps))
}

pub fn calculate_spot_fee_amount(swap_data: &SwapData, to_asset: &Asset, fee_rate: f64, builder_fee_bps: u32) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
    let builder_rate = if to_asset.id == *HYPERCORE_SPOT_USDC_ASSET_ID { builder_fee_rate(builder_fee_bps) } else { 0.0 };
    let received = BigNumberFormatter::value_as_f64(&swap_data.quote.to_value, 0)? / (1.0 - builder_rate);

    Ok(BigInt::from((received * (fee_rate + builder_rate)) as i64))
}

fn fee_amount_in_usdc(value: f64, fee_rate: f64) -> BigInt {
    BigInt::from((value * fee_rate * USDC_DECIMALS_MULTIPLIER) as i64)
}

pub(crate) fn builder_fee_rate(builder_fee_bps: u32) -> f64 {
    f64::from(builder_fee_bps) / f64::from(BUILDER_FEE_RATE_SCALE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{
        SwapProvider,
        known_assets::{HYPERCORE_HYPE, HYPERCORE_SPOT_USDC},
    };

    #[test]
    fn calculate_perpetual_fee_amount_cases() {
        for (fiat_value, fee_rate, builder_fee_bps, expected) in [
            (100.0, 0.00045, 0, 45_000_i64),
            (1000.0, 0.000315, 0, 315_000),
            (1000.0, 0.0, 0, 0),
            (0.0, 0.000025, 45, 0),
            (1.0, 0.000043, 0, 43),
            (10000.0, 0.001, 0, 10_000_000),
            (10000.0, 0.00045, 45, 9_000_000),
            (100.0, 0.0, 45, 45_000),
        ] {
            assert_eq!(calculate_perpetual_fee_amount(fiat_value, fee_rate, builder_fee_bps), BigInt::from(expected));
        }
    }

    #[test]
    fn test_spot_fee_is_charged_in_the_received_token() {
        let sell = SwapData::mock_with_values(SwapProvider::Hyperliquid, "30000000", "1181917897");
        let buy = SwapData::mock_with_values(SwapProvider::Hyperliquid, "1197900000", "29986500");

        assert_eq!(
            calculate_spot_fee_amount(&sell, &HYPERCORE_SPOT_USDC, 0.00056, 45).unwrap(),
            BigInt::from(1_194_274),
            "a sell pays the trade and builder fees in USDC"
        );
        assert_eq!(calculate_spot_fee_amount(&buy, &HYPERCORE_HYPE, 0.00056, 45).unwrap(), BigInt::from(16_792), "a buy pays only the trade fee, in HYPE");
    }
}
