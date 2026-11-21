use std::fmt;

use gem_url::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentDecoderError {
    InvalidScheme,
    InvalidFormat(String),
    MissingField(String),
    UrlParse(String),
}

impl fmt::Display for PaymentDecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentDecoderError::InvalidScheme => write!(f, "Invalid or unsupported scheme"),
            PaymentDecoderError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            PaymentDecoderError::MissingField(field) => write!(f, "Missing field: {}", field),
            PaymentDecoderError::UrlParse(msg) => write!(f, "URL parse error: {}", msg),
        }
    }
}

impl std::error::Error for PaymentDecoderError {}

impl From<ParseError> for PaymentDecoderError {
    fn from(err: ParseError) -> Self {
        PaymentDecoderError::UrlParse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PaymentDecoderError>;
