use std::fmt;

use gem_client::ClientError;

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentError {
    NoPaymentOptions,
    InvalidRequest { reason: String },
    Network { reason: String },
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPaymentOptions => write!(f, "No payment options"),
            Self::InvalidRequest { reason } | Self::Network { reason } => write!(f, "{reason}"),
        }
    }
}

impl std::error::Error for PaymentError {}

impl From<ClientError> for PaymentError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Http { status, .. } => Self::Network {
                reason: format!("Payment gateway returned HTTP {status}"),
            },
            ClientError::Network(reason) | ClientError::Serialization(reason) => Self::Network { reason },
            ClientError::Timeout => Self::Network {
                reason: "Payment gateway request timed out".to_string(),
            },
        }
    }
}
