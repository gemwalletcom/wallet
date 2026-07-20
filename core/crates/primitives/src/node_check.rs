use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsRefStr, EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum NodeCheckProfile {
    LoadBalancer,
    Parser,
    ArchivalParser,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NodeCheckStatus {
    Passed { result: String },
    Failed { error: String },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCheckReport {
    pub checks: BTreeMap<String, NodeCheckStatus>,
}

impl NodeCheckReport {
    pub fn is_healthy(&self) -> bool {
        !self.checks.is_empty()
            && self.checks.values().all(|status| match status {
                NodeCheckStatus::Passed { .. } => true,
                NodeCheckStatus::Failed { .. } => false,
            })
    }

    pub fn error(&self) -> Option<String> {
        self.checks.iter().find_map(|(method, status)| match status {
            NodeCheckStatus::Passed { .. } => None,
            NodeCheckStatus::Failed { error } => Some(format!("{method}: {error}")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_check_report() {
        assert_eq!(serde_json::from_str::<NodeCheckProfile>(r#""load_balancer""#).unwrap(), NodeCheckProfile::LoadBalancer);
        assert_eq!(serde_json::from_str::<NodeCheckProfile>(r#""archival_parser""#).unwrap(), NodeCheckProfile::ArchivalParser);

        let report = NodeCheckReport {
            checks: BTreeMap::from([("eth_getBlockByNumber".to_string(), NodeCheckStatus::Passed { result: "22820942".to_string() })]),
        };
        assert!(report.is_healthy());
        assert_eq!(report.error(), None);
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::json!({
                "checks": {
                    "eth_getBlockByNumber": {
                        "status": "passed",
                        "result": "22820942"
                    }
                }
            })
        );

        let report = NodeCheckReport {
            checks: BTreeMap::from([(
                "eth_getBlockByNumber".to_string(),
                NodeCheckStatus::Failed {
                    error: "returned null".to_string(),
                },
            )]),
        };
        assert!(!report.is_healthy());
        assert_eq!(report.error().as_deref(), Some("eth_getBlockByNumber: returned null"));
        assert!(!NodeCheckReport::default().is_healthy());
    }
}
