use crate::config::{ChainConfig, ErrorMatcherConfig, FailureTriggerConfig, NodeMonitoringConfig, RetryConfig, Url};
use crate::jsonrpc_types::RequestType;
use primitives::{Chain, MINUTE, NodeCheckProfile};
use serde_json::json;

pub fn url(url: &str) -> Url {
    Url {
        url: url.to_string(),
        headers: None,
    }
}

pub fn chain_config(chain: Chain, node_url: &str) -> ChainConfig {
    ChainConfig {
        chain,
        poll_interval_seconds: None,
        latency: None,
        overrides: None,
        allowlist: None,
        urls: vec![url(node_url)],
    }
}

pub fn jsonrpc(method: &str) -> RequestType {
    RequestType::from_request(
        "POST",
        "/".to_string(),
        serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": [],
            "id": 1
        }))
        .unwrap(),
    )
}

pub fn monitoring_config() -> NodeMonitoringConfig {
    NodeMonitoringConfig {
        enabled: true,
        profile: NodeCheckProfile::Basic,
        interval: MINUTE * 10,
        trigger: FailureTriggerConfig {
            failures: 15,
            rate: 50,
            window: MINUTE,
            latency: None,
        },
    }
}

pub fn retry_config(enabled: bool, status_codes: Vec<u16>, error_messages: Vec<&str>) -> RetryConfig {
    retry_config_with_attempts(enabled, 0, status_codes, error_messages)
}

pub fn retry_config_with_attempts(enabled: bool, max_attempts: usize, status_codes: Vec<u16>, error_messages: Vec<&str>) -> RetryConfig {
    RetryConfig {
        enabled,
        max_attempts,
        errors: error_matcher_config(status_codes, error_messages),
    }
}

pub fn error_matcher_config(status_codes: Vec<u16>, error_messages: Vec<&str>) -> ErrorMatcherConfig {
    ErrorMatcherConfig {
        status_codes,
        error_messages: error_messages.into_iter().map(|value| value.to_string()).collect(),
    }
}
