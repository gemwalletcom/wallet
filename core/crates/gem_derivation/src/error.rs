use std::error::Error;
use std::fmt;

use primitives::Chain;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountDerivationError {
    InvalidInput(String),
    InvalidPrivateKey,
    Unsupported(String),
}

impl fmt::Display for AccountDerivationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountDerivationError::InvalidInput(message) => write!(f, "Invalid account derivation input: {}", message),
            AccountDerivationError::InvalidPrivateKey => write!(f, "Invalid private key"),
            AccountDerivationError::Unsupported(message) => write!(f, "Unsupported account derivation: {}", message),
        }
    }
}

impl Error for AccountDerivationError {}

impl AccountDerivationError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }

    pub fn unsupported_chain(chain: Chain) -> Self {
        Self::unsupported(format!("account derivation is not implemented for {}", chain))
    }
}

impl From<primitives::SignerError> for AccountDerivationError {
    fn from(error: primitives::SignerError) -> Self {
        Self::InvalidInput(error.to_string())
    }
}

impl From<gem_keystore::KeystoreError> for AccountDerivationError {
    fn from(error: gem_keystore::KeystoreError) -> Self {
        Self::InvalidInput(error.to_string())
    }
}

impl From<gem_crypto::CryptoError> for AccountDerivationError {
    fn from(error: gem_crypto::CryptoError) -> Self {
        match error {
            gem_crypto::CryptoError::InvalidInput(message) => Self::invalid_input(message),
            gem_crypto::CryptoError::Random => Self::invalid_input(error.to_string()),
        }
    }
}
