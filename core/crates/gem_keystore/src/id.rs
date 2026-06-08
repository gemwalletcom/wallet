use std::fmt;
use std::str::FromStr;

use uuid::{Uuid, Version};

use crate::KeystoreError;

/// Fixed namespace for deriving deterministic v5 keystore ids from a wallet id.
const KEYSTORE_NAMESPACE: Uuid = Uuid::from_bytes(*b"GemKeystoreV4\0\0\0");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeystoreId(String);

impl KeystoreId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_wallet_id(wallet_id: &str) -> Self {
        Self(Uuid::new_v5(&KEYSTORE_NAMESPACE, wallet_id.as_bytes()).to_string())
    }

    pub fn parse(value: &str) -> Result<Self, KeystoreError> {
        let uuid = Uuid::parse_str(value).map_err(|_| KeystoreError::invalid_input("keystore id"))?;
        if !matches!(uuid.get_version(), Some(Version::Random | Version::Sha1)) {
            return Err(KeystoreError::invalid_input("keystore id"));
        }
        if uuid.to_string() != value {
            return Err(KeystoreError::invalid_input("keystore id"));
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Default for KeystoreId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for KeystoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for KeystoreId {
    type Err = KeystoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_id_from_wallet_id_is_deterministic() {
        let wallet_id = "multicoin_0x5a8f70b44aFa00Cb70615D9c9CCb9A24933ED2D3";
        let id = KeystoreId::from_wallet_id(wallet_id);
        assert_eq!(id.as_str(), "f32a9e95-4904-533b-95fe-ebbe6cfb7554");
        assert_eq!(id, KeystoreId::from_wallet_id(wallet_id));
        assert_ne!(id, KeystoreId::from_wallet_id("multicoin_0xother"));
        assert_eq!(KeystoreId::parse(id.as_str()).unwrap(), id);
    }

    #[test]
    fn test_keystore_id_validation() {
        let id = KeystoreId::new();
        assert_eq!(KeystoreId::parse(id.as_str()).unwrap(), id);

        assert_eq!(KeystoreId::parse("").unwrap_err(), KeystoreError::invalid_input("keystore id"));
        assert_eq!(
            KeystoreId::parse("550e8400-e29b-11d4-a716-446655440000").unwrap_err(),
            KeystoreError::invalid_input("keystore id")
        );
        assert_eq!(
            KeystoreId::parse("550E8400-E29B-41D4-A716-446655440000").unwrap_err(),
            KeystoreError::invalid_input("keystore id")
        );
        assert_eq!(
            KeystoreId::parse("../550e8400-e29b-41d4-a716-446655440000").unwrap_err(),
            KeystoreError::invalid_input("keystore id")
        );
        assert_eq!(
            KeystoreId::parse("550e8400-e29b-41d4-a716-446655440000.gemk").unwrap_err(),
            KeystoreError::invalid_input("keystore id")
        );
        assert_eq!(
            KeystoreId::parse("550e8400-e29b-41d4-a716-446655440000%2fsecret").unwrap_err(),
            KeystoreError::invalid_input("keystore id")
        );
    }
}
