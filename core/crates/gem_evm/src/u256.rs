use std::error::Error;

use alloy_primitives::U256;
use num_bigint::{BigInt, BigUint, Sign};

pub fn u256_to_biguint(value: &U256) -> BigUint {
    BigUint::from_bytes_be(&value.to_be_bytes::<32>())
}

pub fn bigint_to_u256(value: &BigInt) -> Result<U256, Box<dyn Error + Send + Sync>> {
    if value.sign() == Sign::Minus {
        return Err("Negative values are not supported".into());
    }
    Ok(U256::from_be_slice(&value.to_bytes_be().1))
}

pub fn biguint_to_u256(value: &BigUint) -> Option<U256> {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        return None;
    }

    Some(U256::from_be_slice(&bytes))
}
