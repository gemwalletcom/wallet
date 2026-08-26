use crate::api::GemApiError;
use crate::services::preferences::GemPreferencesError;

#[derive(Debug, uniffi::Error)]
pub enum GemConfigError {
    Api { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemConfigError {}

impl From<GemApiError> for GemConfigError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemPreferencesError> for GemConfigError {
    fn from(error: GemPreferencesError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
