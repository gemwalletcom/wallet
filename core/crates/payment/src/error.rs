use std::fmt;

use gem_client::ClientError;
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ErrorResponse {
    fn reason(self) -> Option<String> {
        [self.message, self.error].into_iter().find(|reason| !reason.is_empty())
    }
}

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
            ClientError::Http { status, body } => match serde_json::from_slice::<ErrorResponse>(&body).ok().and_then(ErrorResponse::reason) {
                Some(reason) => Self::InvalidRequest { reason },
                None => Self::Network {
                    reason: format!("Payment gateway returned HTTP {status}"),
                },
            },
            ClientError::Network(reason) | ClientError::Serialization(reason) => Self::Network { reason },
            ClientError::Timeout => Self::Network {
                reason: "Payment gateway request timed out".to_string(),
            },
        }
    }
}
