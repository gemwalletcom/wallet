use num_traits::ToPrimitive;
use number_formatter::Decimal;
use primitives::SignerError;

use crate::address::XrpAddress;
use crate::currency::{CURRENCY_CODE_LENGTH, MAX_IOU_PRECISION, currency_code_bytes};

const POS_SIGN_BIT_MASK: u64 = 0x4000_0000_0000_0000;
const ISSUED_CURRENCY_ZERO: u64 = 0x8000_0000_0000_0000;
const MIN_MANTISSA: u128 = 1_000_000_000_000_000;
const MAX_MANTISSA: u128 = 9_999_999_999_999_999;
const MIN_IOU_EXPONENT: i32 = -96;
const MAX_IOU_EXPONENT: i32 = 80;
const MAX_DROPS: u64 = 100_000_000_000_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XrpAmount {
    Native(u64),
    Issued(IssuedAmount),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IssuedAmount {
    value: Decimal,
    currency: [u8; CURRENCY_CODE_LENGTH],
    issuer: XrpAddress,
}

impl XrpAmount {
    pub(crate) fn native(value: &str) -> Result<Self, SignerError> {
        let amount = value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid XRP amount"))?;
        if amount > MAX_DROPS {
            return Err(SignerError::invalid_input("XRP amount is too large"));
        }
        Ok(Self::Native(amount))
    }

    pub(crate) fn issued(value: &str, currency: &str, issuer: &str) -> Result<Self, SignerError> {
        IssuedAmount::new(value, currency, issuer).map(Self::Issued)
    }

    pub(crate) fn encode(&self, buffer: &mut Vec<u8>) -> Result<(), SignerError> {
        match self {
            Self::Native(amount) => buffer.extend_from_slice(&(*amount | POS_SIGN_BIT_MASK).to_be_bytes()),
            Self::Issued(amount) => amount.encode(buffer)?,
        }
        Ok(())
    }
}

impl IssuedAmount {
    fn new(value: &str, currency: &str, issuer: &str) -> Result<Self, SignerError> {
        Ok(Self {
            value: value.parse().map_err(|_| SignerError::invalid_input("invalid XRP token amount"))?,
            currency: currency_code_bytes(currency)?,
            issuer: XrpAddress::parse(issuer)?,
        })
    }

    fn encode(&self, buffer: &mut Vec<u8>) -> Result<(), SignerError> {
        buffer.extend_from_slice(&self.encoded_value()?.to_be_bytes());
        buffer.extend_from_slice(&self.currency);
        buffer.extend_from_slice(self.issuer.as_bytes());
        Ok(())
    }

    fn encoded_value(&self) -> Result<u64, SignerError> {
        if self.value.is_zero() {
            return Ok(ISSUED_CURRENCY_ZERO);
        }

        let value = self.value.normalized();
        if value.digits().magnitude().to_string().len() > MAX_IOU_PRECISION {
            return Err(SignerError::invalid_input("XRP token amount precision is too large"));
        }
        let is_positive = !value.is_negative();
        let mut exponent = i32::try_from(value.scale())
            .ok()
            .and_then(i32::checked_neg)
            .filter(|exponent| (MIN_IOU_EXPONENT..=MAX_IOU_EXPONENT).contains(exponent))
            .ok_or_else(|| SignerError::invalid_input("XRP token amount exponent is out of range"))?;
        let mut mantissa = value.digits().magnitude().to_u128().ok_or_else(|| SignerError::invalid_input("invalid XRP token amount"))?;

        while mantissa < MIN_MANTISSA && exponent > MIN_IOU_EXPONENT {
            mantissa *= 10;
            exponent -= 1;
        }

        while mantissa > MAX_MANTISSA {
            if exponent >= MAX_IOU_EXPONENT {
                return Err(SignerError::invalid_input("XRP token amount is too large"));
            }
            mantissa /= 10;
            exponent += 1;
        }

        if exponent < MIN_IOU_EXPONENT || mantissa < MIN_MANTISSA {
            return Ok(ISSUED_CURRENCY_ZERO);
        }

        if exponent > MAX_IOU_EXPONENT || mantissa > MAX_MANTISSA {
            return Err(SignerError::invalid_input("XRP token amount is too large"));
        }

        let mut encoded = ISSUED_CURRENCY_ZERO;
        if is_positive {
            encoded |= POS_SIGN_BIT_MASK;
        }
        encoded |= ((exponent as i64 + 97) as u64) << 54;
        encoded |= mantissa as u64;
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_issued() {
        let encoded = |value: &str| -> Result<u64, SignerError> {
            let mut buffer = Vec::new();
            XrpAmount::issued(value, "USD", "rf1BiGeXwwQoi8Z2ueFYTEXSwuJYfV2Jpn")?.encode(&mut buffer)?;
            Ok(u64::from_be_bytes(buffer[..8].try_into().unwrap()))
        };
        let out_of_range = Err(SignerError::invalid_input("XRP token amount exponent is out of range"));

        assert_eq!(encoded("0"), Ok(ISSUED_CURRENCY_ZERO));
        assert_eq!(encoded("-0.000"), Ok(ISSUED_CURRENCY_ZERO));
        assert_eq!(encoded("1e-96"), Ok(ISSUED_CURRENCY_ZERO));
        assert_eq!(encoded("10"), Ok(0xd4c38d7ea4c68000));
        assert_eq!(encoded("29.3e-1"), Ok(0xd48a68d1c9312000));
        assert_eq!(encoded("1000"), Ok(0xd5438d7ea4c68000));
        assert_eq!(encoded("-12.5"), Ok(0x94c470de4df82000));
        assert_eq!(encoded("-12.3400"), Ok(0x94c4625103a72000));
        assert_eq!(encoded("0.0001234567890123456"), Ok(0xd38462d53c8abac0));
        assert_eq!(encoded("0.001"), Ok(0xd3c38d7ea4c68000));
        assert_eq!(encoded("1234567890123456"), Ok(0xd84462d53c8abac0));
        assert_eq!(encoded("9999999999999999e80"), Ok(0xec6386f26fc0ffff));
        assert_eq!(encoded("12345678901234567"), Err(SignerError::invalid_input("XRP token amount precision is too large")));
        assert_eq!(encoded("1e-97"), out_of_range);
        assert_eq!(encoded("1e96"), out_of_range);
        assert_eq!(encoded("1e-4294967297"), out_of_range);
        assert_eq!(encoded("1e4294967297"), out_of_range);
        for invalid in ["", "abc", "1_000", ".-5", "1e", "NaN"] {
            assert_eq!(encoded(invalid), Err(SignerError::invalid_input("invalid XRP token amount")), "{invalid}");
        }
    }
}
