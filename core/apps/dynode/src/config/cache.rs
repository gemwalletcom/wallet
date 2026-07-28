use std::collections::HashMap;
use std::time::Duration;

use primitives::{Chain, ChainType};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::duration;

use super::path_without_query;

pub(crate) const ETH_CALL: &str = "eth_call";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CacheConfig {
    pub max_memory_mb: usize,
    #[serde(default)]
    chain_types: HashMap<ChainType, Vec<CacheRule>>,
    #[serde(default)]
    chains: HashMap<Chain, Vec<CacheRule>>,
}

impl CacheConfig {
    pub(crate) fn rules(&self, chain: Chain) -> Vec<CacheRule> {
        self.chain_types
            .get(&chain.chain_type())
            .into_iter()
            .flatten()
            .chain(self.chains.get(&chain).into_iter().flatten())
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CacheRule {
    pub(crate) path: Option<String>,
    pub(crate) method: Option<String>,
    pub(crate) rpc_method: Option<String>,
    pub(crate) contract: Option<String>,
    pub(crate) selector: Option<String>,
    #[serde(default, alias = "ttl_seconds", deserialize_with = "duration::deserialize_option")]
    pub(crate) ttl: Option<Duration>,
    #[serde(default)]
    pub(crate) params: HashMap<String, Value>,
}

impl CacheRule {
    pub(crate) fn matches_path_request(&self, path: &str, method: &str, body: Option<&[u8]>) -> bool {
        let Some(rule_method) = self.method.as_ref() else {
            return false;
        };
        if method != rule_method {
            return false;
        }

        let Some(rule_path) = self.path.as_ref() else {
            return false;
        };
        path_without_query(path) == rule_path && self.matches_body(body)
    }

    pub(crate) fn matches_rpc(&self, rpc_method: &str, params: &Value) -> bool {
        if self.rpc_method.as_deref() != Some(rpc_method) {
            return false;
        }
        if rpc_method != ETH_CALL {
            return true;
        }

        let (Some(contract), Some(selector)) = (&self.contract, &self.selector) else {
            return false;
        };
        matches_evm_call(params, contract, selector)
    }

    fn matches_body(&self, body: Option<&[u8]>) -> bool {
        if self.params.is_empty() {
            return true;
        }

        let Some(body_bytes) = body else {
            return false;
        };

        let Ok(value) = serde_json::from_slice::<Value>(body_bytes) else {
            return false;
        };

        let Some(object) = value.as_object() else {
            return false;
        };

        self.params.iter().all(|(key, expected)| object.get(key) == Some(expected))
    }
}

fn matches_evm_call(params: &Value, contract: &str, selector: &str) -> bool {
    if !is_hex(contract, 42) || !is_hex(selector, 10) {
        return false;
    }
    let Some(params) = params.as_array() else {
        return false;
    };
    if params.len() != 2 || params.get(1).and_then(Value::as_str) != Some("latest") {
        return false;
    }
    let Some(call) = params.first().and_then(Value::as_object) else {
        return false;
    };
    let Some(call_contract) = call.get("to").and_then(Value::as_str) else {
        return false;
    };
    let Some(data) = call.get("data").and_then(Value::as_str) else {
        return false;
    };
    let Some(call_selector) = data.get(..10) else {
        return false;
    };
    call.len() == 2 && is_hex(call_contract, 42) && is_hex(call_selector, 10) && call_contract.eq_ignore_ascii_case(contract) && call_selector.eq_ignore_ascii_case(selector)
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.starts_with("0x") && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::MINUTE;

    const CONTRACT: &str = "0x1111111111111111111111111111111111111111";

    fn evm_call_rule(selector: &str) -> CacheRule {
        serde_json::from_value(serde_json::json!({
            "rpc_method": "eth_call",
            "contract": CONTRACT,
            "selector": selector,
            "ttl": "1h"
        }))
        .unwrap()
    }

    fn eth_call_params(contract: &str, data: &str) -> Value {
        serde_json::json!([{ "to": contract, "data": data }, "latest"])
    }

    #[test]
    fn test_ttl_default_none() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "path": "/wallet/getaccount",
            "method": "POST"
        }))
        .unwrap();

        assert_eq!(rule.ttl, None);
    }

    #[test]
    fn test_ttl_duration_string() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "path": "/api/data",
            "method": "GET",
            "ttl": "1m"
        }))
        .unwrap();

        assert_eq!(rule.ttl, Some(MINUTE));
    }

    #[test]
    fn test_static_evm_call_matching() {
        let rule = evm_call_rule("0x1698ee82");
        let wrong_block = serde_json::json!([{ "to": CONTRACT, "data": "0x1698ee820000" }, "pending"]);

        assert!(rule.matches_rpc(ETH_CALL, &eth_call_params(CONTRACT, "0x1698ee820000")));
        assert!(rule.matches_rpc(ETH_CALL, &eth_call_params(&CONTRACT.to_uppercase().replacen("0X", "0x", 1), "0x1698EE820000")));
        assert!(!rule.matches_rpc("eth_estimateGas", &eth_call_params(CONTRACT, "0x1698ee820000")));
        assert!(!rule.matches_rpc(ETH_CALL, &wrong_block));
        assert!(!rule.matches_rpc(ETH_CALL, &eth_call_params("0x2222222222222222222222222222222222222222", "0x1698ee820000")));
        assert!(!rule.matches_rpc(ETH_CALL, &eth_call_params(CONTRACT, "0xaa9d21cb0000")));
        assert!(!rule.matches_rpc(
            ETH_CALL,
            &serde_json::json!([
                {
                    "to": CONTRACT,
                    "data": "0x1698ee82",
                    "value": "0x1"
                },
                "latest"
            ])
        ));

        let method_only: CacheRule = serde_json::from_value(serde_json::json!({
            "rpc_method": "eth_call",
            "ttl": "1h"
        }))
        .unwrap();
        let params = eth_call_params(CONTRACT, "0x1698ee82");

        assert!(!method_only.matches_rpc(ETH_CALL, &params));
        let invalid_selector: CacheRule = serde_json::from_value(serde_json::json!({
            "rpc_method": "eth_call",
            "contract": CONTRACT,
            "selector": "0x1698",
            "ttl": "1h"
        }))
        .unwrap();
        assert!(!invalid_selector.matches_rpc(ETH_CALL, &params));
    }

    #[test]
    fn test_legacy_rpc_rule_compatibility() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "rpc_method": "eth_chainId",
            "ttl": "1h"
        }))
        .unwrap();

        assert!(rule.matches_rpc("eth_chainId", &Value::Null));
        assert!(!rule.matches_rpc("eth_blockNumber", &Value::Null));
    }
}
