use num_bigint::BigUint;
use primitives::{SignerError, hex::decode_hex_array};

pub(crate) const CURRENCY_CODE_LENGTH: usize = 20;
pub(crate) const MAX_IOU_PRECISION: usize = 16;
const STANDARD_CURRENCY_CODE_LENGTH: usize = 3;
const STANDARD_CURRENCY_CODE_OFFSET: usize = 12;
const XRP_CURRENCY: &str = "XRP";

pub(crate) fn currency_code_bytes(value: &str) -> Result<[u8; CURRENCY_CODE_LENGTH], SignerError> {
    if value == XRP_CURRENCY {
        return Ok([0; CURRENCY_CODE_LENGTH]);
    }
    if value.len() == CURRENCY_CODE_LENGTH * 2 || value.starts_with("0x") {
        return decode_hex_array(value).map_err(|_| SignerError::invalid_input("invalid XRP currency code"));
    }

    let mut bytes = [0; CURRENCY_CODE_LENGTH];
    let value = value.as_bytes();
    if value.len() > CURRENCY_CODE_LENGTH {
        return Err(SignerError::invalid_input("XRP currency symbol is too long"));
    }
    if value.len() == STANDARD_CURRENCY_CODE_LENGTH {
        let end = STANDARD_CURRENCY_CODE_OFFSET + STANDARD_CURRENCY_CODE_LENGTH;
        bytes[STANDARD_CURRENCY_CODE_OFFSET..end].copy_from_slice(value);
        return Ok(bytes);
    }
    bytes[..value.len()].copy_from_slice(value);
    Ok(bytes)
}

pub fn currency_code(symbol: &str) -> Result<String, SignerError> {
    Ok(hex::encode_upper(currency_code_bytes(symbol)?))
}

pub fn truncate_to_iou_precision(value: &BigUint) -> BigUint {
    let digits = value.to_string().len();
    match digits.checked_sub(MAX_IOU_PRECISION) {
        Some(excess) if excess > 0 => {
            let scale = BigUint::from(10u32).pow(excess as u32);
            value / &scale * scale
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currency_bytes() {
        assert_eq!(hex::encode(currency_code_bytes("USD").unwrap()), "0000000000000000000000005553440000000000");
        assert_eq!(hex::encode(currency_code_bytes("524C555344000000000000000000000000000000").unwrap()), "524c555344000000000000000000000000000000");
        assert_eq!(hex::encode(currency_code_bytes("RLUSD").unwrap()), "524c555344000000000000000000000000000000");
        assert_eq!(currency_code_bytes("XRP").unwrap(), [0; CURRENCY_CODE_LENGTH]);
    }

    #[test]
    fn test_currency_code() {
        assert_eq!(currency_code("RLUSD").unwrap(), "524C555344000000000000000000000000000000");
        assert_eq!(currency_code("USDC").unwrap(), "5553444300000000000000000000000000000000");
    }

    #[test]
    fn test_truncate_to_iou_precision() {
        assert_eq!(truncate_to_iou_precision(&BigUint::from(1_234_567_890_123_456u64)), BigUint::from(1_234_567_890_123_456u64));
        assert_eq!(truncate_to_iou_precision(&BigUint::from(12_345_678_901_234_567u64)), BigUint::from(12_345_678_901_234_560u64));
        assert_eq!(truncate_to_iou_precision(&BigUint::from(99_999_999_999_999_999_999u128)), BigUint::from(99_999_999_999_999_990_000u128));
        assert_eq!(truncate_to_iou_precision(&BigUint::ZERO), BigUint::ZERO);
    }
}
