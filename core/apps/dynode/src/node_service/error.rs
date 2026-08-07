use std::{error::Error, fmt};

use primitives::Chain;
use reqwest::StatusCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeServiceError {
    ChainNotConfigured(Chain),
    NodeNotFound,
    RequestNotAllowed,
    UpstreamStatus(u16),
    UpstreamsFailed,
}

impl NodeServiceError {
    pub(super) const fn status(self) -> StatusCode {
        match self {
            Self::RequestNotAllowed => StatusCode::FORBIDDEN,
            Self::ChainNotConfigured(_) | Self::NodeNotFound | Self::UpstreamStatus(_) | Self::UpstreamsFailed => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl fmt::Display for NodeServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChainNotConfigured(chain) => write!(formatter, "Chain {chain} not configured"),
            Self::NodeNotFound => formatter.write_str("Node not found"),
            Self::RequestNotAllowed => formatter.write_str("Request not allowed"),
            Self::UpstreamStatus(status) => write!(formatter, "Upstream status code: {status}"),
            Self::UpstreamsFailed => formatter.write_str("All upstream URLs failed"),
        }
    }
}

impl Error for NodeServiceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_errors_define_stable_messages_and_statuses() {
        assert_eq!(NodeServiceError::RequestNotAllowed.to_string(), "Request not allowed");
        assert_eq!(NodeServiceError::RequestNotAllowed.status(), StatusCode::FORBIDDEN);
        assert_eq!(NodeServiceError::UpstreamsFailed.to_string(), "All upstream URLs failed");
        assert_eq!(NodeServiceError::UpstreamsFailed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
