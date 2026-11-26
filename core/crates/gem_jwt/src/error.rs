use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JwtError {
    InvalidKey,
    InvalidToken,
    InvalidSignature,
    InvalidClaims,
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "invalid JWT key"),
            Self::InvalidToken => write!(f, "invalid JWT"),
            Self::InvalidSignature => write!(f, "invalid JWT signature"),
            Self::InvalidClaims => write!(f, "invalid JWT claims"),
        }
    }
}

impl Error for JwtError {}
