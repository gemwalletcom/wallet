use crate::api::GemApiError;

#[derive(Debug, uniffi::Error)]
pub enum GemPriceError {
    Api { msg: String },
    Store { msg: String },
    UnknownCurrency { currency: String },
}

impl std::fmt::Display for GemPriceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
            Self::UnknownCurrency { currency } => write!(f, "unknown currency: {currency}"),
        }
    }
}

impl std::error::Error for GemPriceError {}

impl From<GemApiError> for GemPriceError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}
