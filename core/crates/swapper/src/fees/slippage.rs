use num_traits::ToPrimitive;
use number_formatter::{BigNumberFormatter, NumberFormatterError};

pub use gem_evm::slippage::{BasisPointConvert, subtract_bps};

const BPS_PER_PERCENT_DECIMALS: i32 = 2;

pub fn bps_to_percent_string(bps: u32) -> Result<String, NumberFormatterError> {
    BigNumberFormatter::value(&bps.to_string(), BPS_PER_PERCENT_DECIMALS)
}

pub fn percent_to_bps(percent: f64) -> Option<u32> {
    (percent * 10f64.powi(BPS_PER_PERCENT_DECIMALS)).round().to_u32()
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

    #[test]
    fn test_percent_to_bps() {
        assert_eq!(percent_to_bps(0.5), Some(50));
        assert_eq!(percent_to_bps(1.0), Some(100));
        assert_eq!(percent_to_bps(0.57), Some(57));
        assert_eq!(percent_to_bps(0.0), Some(0));
        assert_eq!(percent_to_bps(-1.0), None);
        assert_eq!(percent_to_bps(f64::NAN), None);
    }
}
