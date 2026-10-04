use std::error::Error;
use std::fmt;

use fiat::error::FiatQuoteError;
use primitives::RequestError;
use storage::DatabaseError;

#[derive(Debug)]
pub enum FiatServiceError {
    Request(RequestError),
    Quote(FiatQuoteError),
    Storage(DatabaseError),
    Internal(Box<dyn Error + Send + Sync>),
}

impl FiatServiceError {
    pub(crate) fn provider(error: Box<dyn Error + Send + Sync>) -> Self {
        match error.downcast::<FiatQuoteError>() {
            Ok(error) => Self::Quote(*error),
            Err(error) => Self::Internal(error),
        }
    }
}

impl fmt::Display for FiatServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(error) => write!(f, "{error}"),
            Self::Quote(error) => write!(f, "{error}"),
            Self::Storage(error) => write!(f, "{error}"),
            Self::Internal(error) => write!(f, "{error}"),
        }
    }
}

impl Error for FiatServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(_) | Self::Quote(_) => None,
            Self::Storage(error) => Some(error),
            Self::Internal(error) => Some(error.as_ref()),
        }
    }
}

impl From<RequestError> for FiatServiceError {
    fn from(error: RequestError) -> Self {
        Self::Request(error)
    }
}

impl From<FiatQuoteError> for FiatServiceError {
    fn from(error: FiatQuoteError) -> Self {
        Self::Quote(error)
    }
}

impl From<DatabaseError> for FiatServiceError {
    fn from(error: DatabaseError) -> Self {
        Self::Storage(error)
    }
}

impl From<Box<dyn Error + Send + Sync>> for FiatServiceError {
    fn from(error: Box<dyn Error + Send + Sync>) -> Self {
        Self::Internal(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_error_keeps_quote_errors() {
        assert!(matches!(FiatServiceError::provider(Box::new(FiatQuoteError::InvalidWebhook)), FiatServiceError::Quote(FiatQuoteError::InvalidWebhook)));
        assert!(matches!(FiatServiceError::provider("timeout".into()), FiatServiceError::Internal(error) if error.to_string() == "timeout"));
    }
}
