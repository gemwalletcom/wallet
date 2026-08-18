use std::{error::Error, fmt};

use strum::AsRefStr;

use crate::failure_reason::FailureReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub(super) enum NodeMonitorError {
    Upstream(FailureReason),
    Request,
    NodeCheck,
}

impl NodeMonitorError {
    pub(super) fn from_error(error: &(dyn Error + Send + Sync + 'static)) -> Self {
        match FailureReason::from_error(error) {
            reason @ (FailureReason::Status(_) | FailureReason::Timeout | FailureReason::ConnectError) => Self::Upstream(reason),
            FailureReason::RequestError => Self::Request,
        }
    }

    fn reason(&self) -> FailureReason {
        match self {
            Self::Upstream(reason) => *reason,
            Self::Request | Self::NodeCheck => FailureReason::RequestError,
        }
    }
}

impl fmt::Display for NodeMonitorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reason().fmt(formatter)
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum NodeSwitchReason {
    BlockHeight,
    CurrentNodeError { error: NodeMonitorError, message: String },
    Latency,
    PreferredNode,
}

impl NodeSwitchReason {
    pub(super) fn metric_reason(&self) -> String {
        match self {
            Self::BlockHeight => "block_height".to_string(),
            Self::CurrentNodeError { error, .. } => error.to_string(),
            Self::Latency => "latency".to_string(),
            Self::PreferredNode => "preferred_node".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::ClientError;

    #[test]
    fn metric_reason_categorizes_current_node_errors() {
        let cases = [
            (NodeMonitorError::Upstream(FailureReason::Timeout), "timeout"),
            (NodeMonitorError::Upstream(FailureReason::Status(429)), "status=429"),
            (NodeMonitorError::Upstream(FailureReason::ConnectError), "connect_error"),
            (NodeMonitorError::Request, "request_error"),
            (NodeMonitorError::NodeCheck, "request_error"),
        ];

        for (error, expected) in cases {
            assert_eq!(
                NodeSwitchReason::CurrentNodeError {
                    error,
                    message: "error detail".to_string()
                }
                .metric_reason(),
                expected
            );
        }
        assert_eq!(NodeSwitchReason::PreferredNode.metric_reason(), "preferred_node");
        assert_eq!(NodeSwitchReason::Latency.metric_reason(), "latency");
    }

    #[test]
    fn current_node_error_uses_shared_failure_reason() {
        let cases = [
            (ClientError::Timeout, NodeMonitorError::Upstream(FailureReason::Timeout)),
            (ClientError::Http { status: 503, body: Vec::new() }, NodeMonitorError::Upstream(FailureReason::Status(503))),
            (ClientError::Network("request failed".to_string()), NodeMonitorError::Request),
        ];

        for (error, expected) in cases {
            assert_eq!(NodeMonitorError::from_error(&error), expected);
        }
    }
}
