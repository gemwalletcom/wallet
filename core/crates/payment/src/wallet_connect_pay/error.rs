use gem_client::ClientError;
use std::fmt;

const CODE_PAYMENT_NOT_FOUND: &str = "payment_not_found";
const CODE_PAYMENT_EXPIRED: &str = "payment_expired";
const CODE_QUOTE_EXPIRED: &str = "quote_expired";
const CODE_RATE_LIMITED: &str = "rate_limited";
const CODE_SANCTIONED_USER: &str = "sanctioned_user";
const STALE_OPTION_MESSAGE: &str = "Option not found";

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectPayError {
    PaymentNotFound,
    PaymentExpired,
    NoPaymentOptions,
    UnsupportedAccounts,
    QuoteExpired,
    RateLimited,
    SanctionedUser,
    InvalidRequest(String),
    Unsupported(String),
    Network(String),
}

impl fmt::Display for WalletConnectPayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PaymentNotFound => write!(f, "Payment not found"),
            Self::PaymentExpired => write!(f, "Payment expired"),
            Self::NoPaymentOptions => write!(f, "No payment options available"),
            Self::UnsupportedAccounts => write!(f, "Wallet has no account the payment accepts"),
            Self::QuoteExpired => write!(f, "Payment quote expired"),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::SanctionedUser => write!(f, "Payment not allowed"),
            Self::InvalidRequest(msg) => write!(f, "Invalid payment request: {msg}"),
            Self::Unsupported(msg) => write!(f, "Unsupported payment action: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl std::error::Error for WalletConnectPayError {}

impl From<ClientError> for WalletConnectPayError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Http { status, body } => Self::from_response(status, &body),
            ClientError::Network(msg) | ClientError::Serialization(msg) => Self::Network(msg),
            ClientError::Timeout => Self::Network("Timeout".to_string()),
        }
    }
}

impl WalletConnectPayError {
    fn from_response(status: u16, body: &[u8]) -> Self {
        let body = String::from_utf8_lossy(&body[..body.len().min(512)]).to_string();
        match Self::error_code(&body).as_deref() {
            Some(CODE_PAYMENT_NOT_FOUND) => Self::PaymentNotFound,
            Some(CODE_PAYMENT_EXPIRED) => Self::PaymentExpired,
            Some(CODE_QUOTE_EXPIRED) => Self::QuoteExpired,
            Some(CODE_RATE_LIMITED) => Self::RateLimited,
            Some(CODE_SANCTIONED_USER) => Self::SanctionedUser,
            _ if body.contains(STALE_OPTION_MESSAGE) => Self::QuoteExpired,
            _ => Self::from_status(status, body),
        }
    }

    fn from_status(status: u16, body: String) -> Self {
        match status {
            404 => Self::PaymentNotFound,
            409 => Self::QuoteExpired,
            410 => Self::PaymentExpired,
            429 => Self::RateLimited,
            400 | 422 => Self::InvalidRequest(format!("{status}: {body}")),
            _ => Self::Network(format!("{status}: {body}")),
        }
    }

    fn error_code(body: &str) -> Option<String> {
        serde_json::from_str::<serde_json::Value>(body).ok()?.get("code")?.as_str().map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_client_error() {
        let stale = ClientError::Http {
            status: 400,
            body: br#"{"code":"params_validation","message":"Validation error: Option not found"}"#.to_vec(),
        };
        assert_eq!(WalletConnectPayError::from(stale), WalletConnectPayError::QuoteExpired);

        let invalid = ClientError::Http {
            status: 400,
            body: br#"{"message":"Invalid URL"}"#.to_vec(),
        };
        assert!(matches!(WalletConnectPayError::from(invalid), WalletConnectPayError::InvalidRequest(_)));

        assert_eq!(
            WalletConnectPayError::from(ClientError::Http { status: 410, body: vec![] }),
            WalletConnectPayError::PaymentExpired
        );
    }

    #[test]
    fn test_error_reads_the_gateway_code_over_the_status() {
        let sanctioned = ClientError::Http {
            status: 400,
            body: br#"{"code":"sanctioned_user","message":"User is sanctioned"}"#.to_vec(),
        };
        assert_eq!(WalletConnectPayError::from(sanctioned), WalletConnectPayError::SanctionedUser);

        let not_found = ClientError::Http {
            status: 400,
            body: br#"{"code":"payment_not_found","message":"No such payment"}"#.to_vec(),
        };
        assert_eq!(WalletConnectPayError::from(not_found), WalletConnectPayError::PaymentNotFound);

        let expired = ClientError::Http {
            status: 500,
            body: br#"{"code":"payment_expired"}"#.to_vec(),
        };
        assert_eq!(WalletConnectPayError::from(expired), WalletConnectPayError::PaymentExpired);
    }
}
