use std::{fmt, str::FromStr};

use primitives::hex::{decode_hex, encode_with_0x};
use serde::{Deserialize, Serialize};

use crate::SuiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address([u8; Address::LENGTH]);

impl Address {
    pub const LENGTH: usize = 32;
    pub const ZERO: Self = Self([0; Self::LENGTH]);

    pub const fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SuiError> {
        bytes.try_into().map(Self).map_err(|_| SuiError::invalid_input(format!("Sui address must be {} bytes", Self::LENGTH)))
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub const fn into_inner(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl FromStr for Address {
    type Err = SuiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes = decode_hex(value).map_err(SuiError::from_display)?;
        if value != value.trim() || bytes.is_empty() || bytes.len() > Self::LENGTH {
            return Err(SuiError::invalid_input(format!("Sui address must be 1 to {} bytes", Self::LENGTH)));
        }
        let mut address = [0; Self::LENGTH];
        address[Self::LENGTH - bytes.len()..].copy_from_slice(&bytes);
        Ok(Self(address))
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&encode_with_0x(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_from_str() {
        let parsed = |value: &str| Address::from_str(value).map(|address| address.to_string()).ok();
        let two = Some("0x0000000000000000000000000000000000000000000000000000000000000002".to_string());
        let abc = Some("0x0000000000000000000000000000000000000000000000000000000000000abc".to_string());

        assert_eq!(parsed("0x2"), two);
        assert_eq!(parsed("2"), two);
        assert_eq!(parsed("abc"), abc);
        assert_eq!(parsed("0xABC"), abc);
        assert_eq!(parsed(&"f".repeat(64)), Some(format!("0x{}", "f".repeat(64))));

        for invalid in ["", "0x", "0X2", "0x0g", " 0x2", "0x2 ", "0x0x2", "0x 2", &"f".repeat(65), &format!("0{}", "f".repeat(64))] {
            assert_eq!(parsed(invalid), None, "{invalid}");
        }
    }

    #[test]
    fn test_address_bcs() {
        let address = Address::from_str("0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6").unwrap();
        let bytes = bcs::to_bytes(&address).unwrap();

        assert_eq!(bytes, address.as_bytes());
        assert_eq!(bcs::from_bytes::<Address>(&bytes).unwrap(), address);
        assert_eq!(Address::from_bytes(&bytes).unwrap(), address);
        assert!(Address::from_bytes(&bytes[1..]).is_err());
    }
}
