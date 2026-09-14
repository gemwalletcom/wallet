use std::{error::Error, fmt, result};

use primitives::SignerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolanaError {
    InvalidInput(String),
    InvalidMessage,
}

impl fmt::Display for SolanaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => formatter.write_str(message),
            Self::InvalidMessage => write!(formatter, "Invalid message"),
        }
    }
}

impl Error for SolanaError {}

impl SolanaError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}

impl From<SolanaError> for SignerError {
    fn from(error: SolanaError) -> Self {
        Self::invalid_input(error.to_string())
    }
}

pub type Result<T> = result::Result<T, SolanaError>;
