use gem_client::ClientError;

#[derive(Debug, uniffi::Error)]
pub enum GemApiError {
    Network { msg: String },
    Timeout,
    Http { status: u16, msg: String },
    Serialization { msg: String },
}

impl std::fmt::Display for GemApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network { msg } | Self::Serialization { msg } => write!(f, "{msg}"),
            Self::Timeout => write!(f, "request timed out"),
            Self::Http { status, msg } => write!(f, "{status}: {msg}"),
        }
    }
}

impl std::error::Error for GemApiError {}

impl From<ClientError> for GemApiError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Network(msg) => Self::Network { msg },
            ClientError::Timeout => Self::Timeout,
            ClientError::Http { status, body } => Self::Http {
                status,
                msg: String::from_utf8_lossy(&body).to_string(),
            },
            ClientError::Serialization(msg) => Self::Serialization { msg },
        }
    }
}
