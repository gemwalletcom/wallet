use bigdecimal::{BigDecimal, num_bigint::BigInt as DecimalBigInt};
use num_bigint::BigInt;

pub struct EtherConv {}

impl EtherConv {
    pub fn one() -> BigInt {
        BigInt::from(10u64.pow(18))
    }

    /// Parse Ether to Wei as BigInt
    pub fn parse_ether(ether: &str) -> BigInt {
        to_bn_wei(ether, 18)
    }

    pub fn to_gwei(wei: &BigInt) -> String {
        let gwei_value = BigDecimal::from_bigint(to_decimal_bigint(wei), 0) / BigDecimal::from(10u64.pow(9));
        gwei_value.to_string()
    }
}

pub fn to_bn_wei(value: &str, decimals: u32) -> BigInt {
    let ether_value = value.parse::<BigDecimal>().unwrap();
    let wei_value = (&ether_value * BigDecimal::from(10u64.pow(decimals))).with_scale(0);

    to_bigint(&wei_value.as_bigint_and_exponent().0)
}

fn to_decimal_bigint(value: &BigInt) -> DecimalBigInt {
    DecimalBigInt::from_signed_bytes_be(&value.to_signed_bytes_be())
}

fn to_bigint(value: &DecimalBigInt) -> BigInt {
    BigInt::from_signed_bytes_be(&value.to_signed_bytes_be())
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
    fn test_bigint_version_conversion() {
        let values = [
            BigInt::from(0),
            BigInt::from(1),
            BigInt::from(-1),
            BigInt::parse_bytes(b"115792089237316195423570985008687907853269984665640564039457584007913129639935", 10).unwrap(),
        ];

        for value in values {
            assert_eq!(to_bigint(&to_decimal_bigint(&value)), value);
        }
    }
}
