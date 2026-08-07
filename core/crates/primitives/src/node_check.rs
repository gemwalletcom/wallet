use std::time::Duration;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NodeCheckProfile {
    #[default]
    Basic,
    Wallet,
    Parser,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeCheckRequest {
    Basic,
    Wallet { address: String, transaction_id: Option<String> },
    Parser,
}

impl NodeCheckRequest {
    pub fn profile(&self) -> NodeCheckProfile {
        match self {
            Self::Basic => NodeCheckProfile::Basic,
            Self::Wallet { .. } => NodeCheckProfile::Wallet,
            Self::Parser => NodeCheckProfile::Parser,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NodeCheckStatus {
    Passed { result: String },
    Warning { warning: String },
    Failed { error: String },
}

impl NodeCheckStatus {
    pub fn message(&self) -> &str {
        match self {
            Self::Passed { result } => result,
            Self::Warning { warning } => warning,
            Self::Failed { error } => error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCheckResult {
    pub method: String,
    #[serde(flatten)]
    pub status: NodeCheckStatus,
    pub latency_ms: u64,
}

impl NodeCheckResult {
    pub fn new(method: impl Into<String>, status: NodeCheckStatus, latency: Duration) -> Self {
        Self {
            method: method.into(),
            status,
            latency_ms: latency.as_millis().try_into().unwrap_or(u64::MAX),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCheckReport {
    pub checks: Vec<NodeCheckResult>,
}

impl NodeCheckReport {
    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|result| match result.status {
            NodeCheckStatus::Passed { .. } | NodeCheckStatus::Warning { .. } => true,
            NodeCheckStatus::Failed { .. } => false,
        })
    }

    pub fn error(&self) -> Option<String> {
        self.checks.iter().find_map(|result| match &result.status {
            NodeCheckStatus::Passed { .. } | NodeCheckStatus::Warning { .. } => None,
            NodeCheckStatus::Failed { error } => Some(format!("{}: {error}", result.method)),
        })
    }

    pub fn get(&self, method: &str) -> Option<&NodeCheckResult> {
        self.checks.iter().find(|result| result.method == method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_check_report() {
        let report = NodeCheckReport {
            checks: vec![NodeCheckResult::new(
                "method",
                NodeCheckStatus::Passed { result: "22820942".to_string() },
                Duration::from_millis(42),
            )],
        };
        assert!(report.is_healthy());
        assert_eq!(report.error(), None);
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::json!({
                "checks": [{
                    "method": "method",
                        "status": "passed",
                        "result": "22820942",
                        "latency_ms": 42
                }]
            })
        );

        let report = NodeCheckReport {
            checks: vec![NodeCheckResult::new(
                "optional_method",
                NodeCheckStatus::Warning {
                    warning: "method not found".to_string(),
                },
                Duration::from_millis(7),
            )],
        };
        assert!(report.is_healthy());
        assert_eq!(report.error(), None);
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::json!({
                "checks": [{
                    "method": "optional_method",
                        "status": "warning",
                        "warning": "method not found",
                        "latency_ms": 7
                }]
            })
        );

        let report = NodeCheckReport {
            checks: vec![NodeCheckResult::new(
                "method",
                NodeCheckStatus::Failed {
                    error: "returned null".to_string(),
                },
                Duration::from_millis(3),
            )],
        };
        assert!(!report.is_healthy());
        assert_eq!(report.error().as_deref(), Some("method: returned null"));
    }
}
