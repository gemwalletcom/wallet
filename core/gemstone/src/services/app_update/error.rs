use crate::services::config::GemConfigError;
use crate::services::preferences::GemPreferencesError;

#[derive(Debug, uniffi::Error)]
pub enum GemAppUpdateError {
    Config { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemAppUpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemAppUpdateError {}

impl From<GemConfigError> for GemAppUpdateError {
    fn from(error: GemConfigError) -> Self {
        Self::Config { msg: error.to_string() }
    }
}

impl From<GemPreferencesError> for GemAppUpdateError {
    fn from(error: GemPreferencesError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
