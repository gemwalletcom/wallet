use std::fmt;

#[derive(Debug, Clone)]
pub enum FiatQuoteError {
    MinimumAmount(f64),
    UnsupportedState(String),
    InvalidRequest(String),
    InvalidWebhook,
    RegionUnavailable,
}

impl fmt::Display for FiatQuoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinimumAmount(amount) => write!(f, "Minimum Amount is {}", amount),
            Self::UnsupportedState(state) => write!(f, "Unsupported state: {}", state),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            Self::InvalidWebhook => write!(f, "Invalid webhook payload"),
            Self::RegionUnavailable => write!(f, "Not available in your region"),
        }
    }
}

impl std::error::Error for FiatQuoteError {}
