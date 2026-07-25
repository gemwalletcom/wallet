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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RetryReason {
    Status(u16),
    Timeout,
    ConnectError,
    RequestError,
}

impl RetryReason {
    pub(super) fn from_error(error: &(dyn Error + Send + Sync + 'static)) -> Self {
        match error.downcast_ref::<reqwest::Error>() {
            Some(error) if error.is_timeout() => Self::Timeout,
            Some(error) if error.is_connect() => Self::ConnectError,
            Some(_) | None => Self::RequestError,
        }
    }
}

impl fmt::Display for RetryReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status(status) => write!(formatter, "status={status}"),
            Self::Timeout => formatter.write_str("timeout"),
            Self::ConnectError => formatter.write_str("connect_error"),
            Self::RequestError => formatter.write_str("request_error"),
        }
    }
}

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

    #[test]
    fn retry_reasons_define_stable_metric_labels() {
        assert_eq!(RetryReason::Status(429).to_string(), "status=429");
        assert_eq!(RetryReason::Timeout.to_string(), "timeout");
        assert_eq!(RetryReason::ConnectError.to_string(), "connect_error");
        assert_eq!(RetryReason::RequestError.to_string(), "request_error");
    }
}
