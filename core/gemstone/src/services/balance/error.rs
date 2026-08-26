use crate::gateway::GatewayError;

#[derive(Debug, uniffi::Error)]
pub enum GemBalanceError {
    Gateway { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemBalanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gateway { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemBalanceError {}

impl From<GatewayError> for GemBalanceError {
    fn from(error: GatewayError) -> Self {
        Self::Gateway { msg: error.to_string() }
    }
}
