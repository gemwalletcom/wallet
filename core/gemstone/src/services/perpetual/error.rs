use crate::gateway::GatewayError;
use crate::services::price::GemPriceError;

#[derive(Debug, uniffi::Error)]
pub enum GemPerpetualError {
    Gateway { msg: String },
    Store { msg: String },
    Price { msg: String },
}

impl std::fmt::Display for GemPerpetualError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gateway { msg } | Self::Store { msg } | Self::Price { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemPerpetualError {}

impl From<GatewayError> for GemPerpetualError {
    fn from(error: GatewayError) -> Self {
        Self::Gateway { msg: error.to_string() }
    }
}

impl From<GemPriceError> for GemPerpetualError {
    fn from(error: GemPriceError) -> Self {
        Self::Price { msg: error.to_string() }
    }
}
