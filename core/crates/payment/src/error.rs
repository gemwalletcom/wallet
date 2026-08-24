use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentError {
    NotSupported,
    PaymentNotFound,
    PaymentExpired,
    QuoteExpired,
    NoPaymentOptions,
    Rejected,
    InvalidRequest(String),
    Network(String),
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSupported => write!(f, "Payment is not supported"),
            Self::PaymentNotFound => write!(f, "Payment not found"),
            Self::PaymentExpired => write!(f, "Payment expired"),
            Self::QuoteExpired => write!(f, "Quote expired"),
            Self::NoPaymentOptions => write!(f, "No payment options"),
            Self::Rejected => write!(f, "Payment rejected"),
            Self::InvalidRequest(message) | Self::Network(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PaymentError {}
