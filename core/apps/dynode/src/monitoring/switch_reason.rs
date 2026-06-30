use std::{error::Error, fmt};

use gem_client::ClientError;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CurrentNodeErrorKind {
    Timeout,
    Status(u16),
    Serialization(String),
    Unknown,
}

impl CurrentNodeErrorKind {
    pub(crate) fn from_error(error: &(dyn Error + Send + Sync + 'static)) -> Self {
        if let Some(error) = error.downcast_ref::<ClientError>() {
            return Self::from_client_error(error);
        }
        if let Some(error) = error.downcast_ref::<reqwest::Error>() {
            return Self::from_reqwest_error(error);
        }

        Self::Unknown
    }

    fn from_client_error(error: &ClientError) -> Self {
        match error {
            ClientError::Timeout => Self::Timeout,
            ClientError::Http { status, .. } => Self::Status(*status),
            ClientError::Network(_) => Self::Unknown,
            ClientError::Serialization(message) => Self::Serialization(message.replace(' ', "_")),
        }
    }

    fn from_reqwest_error(error: &reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::Timeout;
        }
        if let Some(status) = error.status() {
            return Self::Status(status.as_u16());
        }

        Self::Unknown
    }
}

impl fmt::Display for CurrentNodeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout => write!(f, "timeout"),
            Self::Status(code) => write!(f, "status_{code}"),
            Self::Serialization(message) => write!(f, "serialization_error_{message}"),
            Self::Unknown => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodeSwitchReason {
    BlockHeight { old_block: u64, new_block: u64 },
    Latency { old_latency_ms: u64, new_latency_ms: u64 },
    CurrentNodeError { kind: CurrentNodeErrorKind, message: String },
}

impl NodeSwitchReason {
    pub(crate) fn metric_reason(&self) -> String {
        match self {
            Self::BlockHeight { .. } => "block_height".to_string(),
            Self::Latency { .. } => "latency".to_string(),
            Self::CurrentNodeError { kind, .. } => kind.to_string(),
        }
    }
}

impl fmt::Display for NodeSwitchReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockHeight { old_block, new_block } => write!(f, "block_behind:{}", new_block.saturating_sub(*old_block)),
            Self::Latency { old_latency_ms, new_latency_ms } => write!(f, "latency:{}ms->{}ms", old_latency_ms, new_latency_ms),
            Self::CurrentNodeError { message, .. } => write!(f, "{}", message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_reason_keeps_stable_switch_labels() {
        assert_eq!(NodeSwitchReason::BlockHeight { old_block: 1, new_block: 2 }.metric_reason(), "block_height");
        assert_eq!(
            NodeSwitchReason::Latency {
                old_latency_ms: 400,
                new_latency_ms: 100
            }
            .metric_reason(),
            "latency"
        );
    }

    #[test]
    fn metric_reason_categorizes_current_node_errors() {
        let cases = [
            (CurrentNodeErrorKind::Timeout, "timeout"),
            (CurrentNodeErrorKind::Status(429), "status_429"),
            (
                CurrentNodeErrorKind::Serialization("missing_field_`result`_at_line_1_column_2".to_string()),
                "serialization_error_missing_field_`result`_at_line_1_column_2",
            ),
            (CurrentNodeErrorKind::Unknown, "error"),
        ];

        for (kind, expected) in cases {
            assert_eq!(
                NodeSwitchReason::CurrentNodeError {
                    kind,
                    message: "error detail".to_string()
                }
                .metric_reason(),
                expected
            );
        }
    }

    #[test]
    fn current_node_error_kind_uses_client_error_variant() {
        let cases = [
            (ClientError::Timeout, CurrentNodeErrorKind::Timeout),
            (ClientError::Http { status: 503, body: Vec::new() }, CurrentNodeErrorKind::Status(503)),
            (ClientError::Network("request failed".to_string()), CurrentNodeErrorKind::Unknown),
            (
                ClientError::Serialization("missing field `result` at line 1 column 2".to_string()),
                CurrentNodeErrorKind::Serialization("missing_field_`result`_at_line_1_column_2".to_string()),
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(CurrentNodeErrorKind::from_error(&error), expected);
        }
    }
}
