use std::error::Error;

use alloy_primitives::U256;
use num_bigint::{BigInt, BigUint, Sign};

pub fn u256_to_biguint(value: &U256) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes::<32>())
}

pub fn biguint_to_u256(value: &BigUint) -> Option<U256> {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        return None;
    }

    Some(U256::from_be_slice(&bytes))
}

pub fn bigint_to_u256(value: &BigInt) -> Result<U256, Box<dyn Error + Send + Sync>> {
    if value.sign() == Sign::Minus {
        return Err("Negative values are not supported".into());
    }
    Ok(biguint_to_u256(value.magnitude()).ok_or("Value does not fit in U256")?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_to_u256_rejects_negative_and_overflow() {
        assert_eq!(bigint_to_u256(&BigInt::from(42u32)).unwrap(), U256::from(42u32));
        assert!(bigint_to_u256(&BigInt::from(-1)).is_err());
        assert!(bigint_to_u256(&(BigInt::from(1) << 256)).is_err());
    }
}
