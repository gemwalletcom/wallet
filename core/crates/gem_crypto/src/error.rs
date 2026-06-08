use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoError {
    InvalidInput(String),
    Random,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::InvalidInput(message) => write!(f, "{}", message),
            CryptoError::Random => write!(f, "secure random generation failed"),
        }
    }
}

impl Error for CryptoError {}

impl CryptoError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}
