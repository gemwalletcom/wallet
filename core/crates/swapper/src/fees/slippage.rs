use number_formatter::{BigNumberFormatter, NumberFormatterError};

pub use gem_evm::slippage::{BasisPointConvert, apply_slippage_in_bp};

const BPS_PER_PERCENT_DECIMALS: i32 = 2;

pub fn bps_to_percent_string(bps: u32) -> Result<String, NumberFormatterError> {
    BigNumberFormatter::value(&bps.to_string(), BPS_PER_PERCENT_DECIMALS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bps_to_percent_string() {
        assert_eq!(bps_to_percent_string(100).unwrap(), "1");
        assert_eq!(bps_to_percent_string(50).unwrap(), "0.5");
        assert_eq!(bps_to_percent_string(200).unwrap(), "2");
        assert_eq!(bps_to_percent_string(10).unwrap(), "0.1");
        assert_eq!(bps_to_percent_string(0).unwrap(), "0");
    }
}
