use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

const UNLIMITED_APPROVE_BIT_WIDTHS: [u32; 2] = [160, 256];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ApprovalData {
    pub token: String,
    pub spender: String,
    pub value: String,
    pub is_unlimited: bool,
}

impl ApprovalData {
    pub fn new(token: String, spender: String, value: String) -> Self {
        Self {
            is_unlimited: Self::is_unlimited_value(&value),
            token,
            spender,
            value,
        }
    }

    fn is_unlimited_value(value: &str) -> bool {
        let Ok(value) = value.parse::<BigUint>() else {
            return false;
        };
        UNLIMITED_APPROVE_BIT_WIDTHS.iter().any(|bits| value == (BigUint::from(1u8) << bits) - BigUint::from(1u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_data_reads_an_unlimited_value() {
        let unlimited = |value: &str| ApprovalData::new("0xtoken".to_string(), "0xspender".to_string(), value.to_string()).is_unlimited;

        assert!(unlimited("115792089237316195423570985008687907853269984665640564039457584007913129639935"));
        assert!(unlimited("1461501637330902918203684832716283019655932542975"));

        assert!(!unlimited("1000000"));
        assert!(!unlimited("0"));
        assert!(!unlimited("not a number"));
    }
}
