use gem_encoding::{decode_base58_into, encode_base58};
use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::SuiError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "Vec<u8>", into = "Vec<u8>")]
pub struct Digest([u8; Digest::LENGTH]);

impl Digest {
    pub const LENGTH: usize = 32;
    pub const ZERO: Self = Self([0; Self::LENGTH]);
    const MAX_ENCODED_LENGTH: usize = 44;

    pub const fn new(bytes: [u8; Self::LENGTH]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub const fn into_inner(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl TryFrom<Vec<u8>> for Digest {
    type Error = SuiError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        bytes.try_into().map(Self).map_err(|_| SuiError::invalid_input(format!("Sui digest must be {} bytes", Self::LENGTH)))
    }
}

impl From<Digest> for Vec<u8> {
    fn from(digest: Digest) -> Self {
        digest.0.to_vec()
    }
}

impl FromStr for Digest {
    type Err = SuiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() > Self::MAX_ENCODED_LENGTH {
            return Err(SuiError::invalid_input("Sui digest is too long"));
        }
        let mut bytes = [0; Self::LENGTH];
        let length = decode_base58_into(value, &mut bytes).map_err(SuiError::from_display)?;
        if length != Self::LENGTH {
            return Err(SuiError::invalid_input(format!("Sui digest must be {} bytes", Self::LENGTH)));
        }
        Ok(Self(bytes))
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&encode_base58(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_from_str() {
        let value = "HdfF7hswRuvbXbEXjGjmUCt7gLybhvbPvvK8zZbCqyD8";
        let digest = Digest::from_str(value).unwrap();

        assert_eq!(digest.to_string(), value);
        assert_eq!(Digest::from_str("11111111111111111111111111111111").unwrap(), Digest::ZERO);

        let max = Digest::new([0xff; Digest::LENGTH]);
        assert_eq!(Digest::from_str(&max.to_string()).unwrap(), max);

        for invalid in ["", "0OIl", "1111111111111111111111111111111", "111111111111111111111111111111111", &"z".repeat(44), &"1".repeat(44), &"z".repeat(100_000)] {
            assert!(Digest::from_str(invalid).is_err(), "{}", invalid.len());
        }
    }

    #[test]
    fn test_digest_bcs() {
        let digest = Digest::new([7; Digest::LENGTH]);
        let bytes = bcs::to_bytes(&digest).unwrap();

        assert_eq!(bytes, [vec![32], vec![7; Digest::LENGTH]].concat());
        assert_eq!(bcs::from_bytes::<Digest>(&bytes).unwrap(), digest);
        assert!(bcs::from_bytes::<Digest>(&[vec![31], vec![7; 31]].concat()).is_err());
    }
}
