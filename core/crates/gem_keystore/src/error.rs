use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeystoreError {
    NotFound,
    AlreadyExists,
    InvalidInput(String),
    Unsupported(String),
    AuthenticationFailed,
    CorruptFile(String),
    Io(String),
}

impl fmt::Display for KeystoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeystoreError::NotFound => write!(f, "Not found"),
            KeystoreError::AlreadyExists => write!(f, "Already exists"),
            KeystoreError::InvalidInput(message) => write!(f, "Invalid {}", message),
            KeystoreError::Unsupported(message) => write!(f, "Unsupported {}", message),
            KeystoreError::AuthenticationFailed => write!(f, "Authentication failed"),
            KeystoreError::CorruptFile(message) => write!(f, "Corrupt keystore file: {}", message),
            KeystoreError::Io(message) => write!(f, "IO error: {}", message),
        }
    }
}

impl Error for KeystoreError {}

impl From<std::io::Error> for KeystoreError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => KeystoreError::NotFound,
            std::io::ErrorKind::AlreadyExists => KeystoreError::AlreadyExists,
            _ => KeystoreError::Io(error.to_string()),
        }
    }
}

impl KeystoreError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::Unsupported(message.into())
    }

    pub fn corrupt_file(message: impl Into<String>) -> Self {
        Self::CorruptFile(message.into())
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self::Io(message.into())
    }
}

#[cfg(any(feature = "mnemonic", feature = "storage"))]
impl From<gem_crypto::CryptoError> for KeystoreError {
    fn from(error: gem_crypto::CryptoError) -> Self {
        match error {
            gem_crypto::CryptoError::InvalidInput(message) => Self::invalid_input(message),
            gem_crypto::CryptoError::Random => Self::io(error.to_string()),
        }
    }
}
