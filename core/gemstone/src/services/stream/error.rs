use std::fmt::Display;

#[derive(Debug, uniffi::Error)]
pub enum GemStreamError {
    Service { msg: String },
}

impl GemStreamError {
    pub fn service(error: impl Display) -> Self {
        Self::Service { msg: error.to_string() }
    }
}

impl std::fmt::Display for GemStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Service { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemStreamError {}
