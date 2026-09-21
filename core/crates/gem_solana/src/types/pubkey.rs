use std::{cmp::Ordering, fmt, str::FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeserializeError};

use crate::{Result as SolanaResult, SolanaError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct Pubkey([u8; 32]);

impl FromStr for Pubkey {
    type Err = SolanaError;

    fn from_str(s: &str) -> SolanaResult<Self> {
        Self::from_base58(s)
    }
}

impl Ord for Pubkey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Pubkey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for Pubkey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_base58())
    }
}

impl<'de> Deserialize<'de> for Pubkey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        Self::from_base58(&s).map_err(DeserializeError::custom)
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl Pubkey {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_base58(s: &str) -> SolanaResult<Self> {
        let bytes = bs58::decode(s).into_vec().map_err(|_| SolanaError::invalid_input(format!("Invalid public key: failed to decode base58: {s}")))?;

        if bytes.len() != 32 {
            return Err(SolanaError::invalid_input(format!("Invalid public key length: {}, expected: 32", bytes.len())));
        }

        let bytes = bytes.try_into().map_err(|bytes: Vec<u8>| SolanaError::invalid_input(format!("Invalid public key length: {}, expected: 32", bytes.len())))?;
        Ok(Self(bytes))
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
