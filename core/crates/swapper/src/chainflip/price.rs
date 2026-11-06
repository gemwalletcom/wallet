use num_bigint::{BigInt, BigUint, Sign};
use number_formatter::f64_parts;

const PRICE_FRACTION_BITS: i32 = 128;

pub fn price_after_slippage(original_price: f64, slippage_bps: u32) -> f64 {
    original_price * (1.0 - slippage_bps as f64 / 10000.0)
}

pub fn price_to_hex_price(price: f64, quote_asset_decimals: u32, base_asset_decimals: u32) -> Result<String, String> {
    if price.is_nan() || price.is_infinite() {
        return Err(format!("Input price ({price}) is NaN or Infinity."));
    }
    let (mantissa, exponent) = f64_parts(price);
    let shift = exponent + PRICE_FRACTION_BITS;
    let numerator = (BigUint::from(mantissa) << shift.max(0) as usize) * BigUint::from(10u32).pow(quote_asset_decimals);
    let denominator = (BigUint::from(1u32) << (-shift).max(0) as usize) * BigUint::from(10u32).pow(base_asset_decimals);
    let sign = if price.is_sign_negative() { Sign::Minus } else { Sign::Plus };
    let hex_price = BigInt::from_biguint(sign, numerator / denominator).to_str_radix(16);
    Ok(format!("0x{}{}", if hex_price.len().is_multiple_of(2) { "" } else { "0" }, hex_price))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_after_slippage_sell() {
        assert_eq!(price_after_slippage(100.0, 100), 99.0);
    }

    #[test]
    fn test_price_to_hex_price() {
        assert_eq!(price_to_hex_price(10000.0, 6, 18).unwrap(), "0x2af31dc4611873bf3f70834acd");
        assert_eq!(price_to_hex_price(3500.12, 6, 18).unwrap(), "0x0f086c320b45b3bc2b00f6de12");
        assert_eq!(price_to_hex_price(0.0001234, 6, 18).unwrap(), "0x08e4540623f437f1acbc");
        assert_eq!(price_to_hex_price(65000.5, 6, 8).unwrap(), "0x028a0147ae147ae147ae147ae147ae147ae1");
        assert_eq!(price_to_hex_price(1.0, 18, 6).unwrap(), "0xe8d4a5100000000000000000000000000000000000");
        assert_eq!(price_to_hex_price(0.5, 8, 8).unwrap(), "0x80000000000000000000000000000000");
        assert_eq!(price_to_hex_price(1e-9, 18, 6).unwrap(), "0x03e800000000001187dc9400000000000000");
        assert_eq!(price_to_hex_price(0.0, 6, 18).unwrap(), "0x00");
        assert_eq!(price_to_hex_price(f64::NAN, 6, 18), Err("Input price (NaN) is NaN or Infinity.".to_string()));
        assert_eq!(price_to_hex_price(f64::INFINITY, 6, 18), Err("Input price (inf) is NaN or Infinity.".to_string()));
    }
}
