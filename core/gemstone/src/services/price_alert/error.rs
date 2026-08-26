use crate::api::GemApiError;
use crate::services::preferences::GemPreferencesError;

#[derive(Debug, uniffi::Error)]
pub enum GemPriceAlertError {
    Api { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemPriceAlertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemPriceAlertError {}

impl From<GemPreferencesError> for GemPriceAlertError {
    fn from(error: GemPreferencesError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}

impl From<GemApiError> for GemPriceAlertError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}
