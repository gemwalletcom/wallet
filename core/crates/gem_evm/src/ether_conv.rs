use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;

const ETHER_DECIMALS: u32 = 18;
const GWEI_DECIMALS: u32 = 9;

pub struct EtherConv {}

impl EtherConv {
    pub fn one() -> BigInt {
        BigInt::from(10u64.pow(ETHER_DECIMALS))
    }

    pub fn parse_ether(ether: &str) -> BigInt {
        to_bn_wei(ether, ETHER_DECIMALS)
    }

    pub fn to_gwei(wei: &BigInt) -> String {
        let unit = BigUint::from(10u64.pow(GWEI_DECIMALS));
        let sign = if wei.sign() == Sign::Minus { "-" } else { "" };
        let whole = wei.magnitude() / &unit;
        let fraction = wei.magnitude() % &unit;
        if fraction.is_zero() {
            return format!("{sign}{whole}");
        }
        let fraction = format!("{fraction:0>width$}", width = GWEI_DECIMALS as usize);
        format!("{sign}{whole}.{}", fraction.trim_end_matches('0'))
    }
}

pub fn to_bn_wei(value: &str, decimals: u32) -> BigInt {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let fraction: String = fraction.chars().chain(std::iter::repeat('0')).take(decimals as usize).collect();
    match format!("{whole}{fraction}").as_str() {
        "" | "-" | "+" => BigInt::zero(),
        digits => BigInt::parse_bytes(digits.as_bytes(), 10).unwrap(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ether_conversion() {
        let ether = "0.0001";
        let wei = EtherConv::parse_ether(ether);

        assert_eq!(wei.to_string(), "100000000000000");

        let ether = "1500.123";
        let wei = EtherConv::parse_ether(ether);

        assert_eq!(wei.to_string(), "1500123000000000000000");
    }

    #[test]
    fn test_to_gwei() {
        assert_eq!(EtherConv::to_gwei(&BigInt::from(0)), "0");
        assert_eq!(EtherConv::to_gwei(&BigInt::from(1_500_000_000u64)), "1.5");
        assert_eq!(EtherConv::to_gwei(&BigInt::from(30_000_000_000u64)), "30");
        assert_eq!(EtherConv::to_gwei(&BigInt::from(1)), "0.000000001");
        assert_eq!(EtherConv::to_gwei(&BigInt::from(-500_000_000)), "-0.5");
    }
}
