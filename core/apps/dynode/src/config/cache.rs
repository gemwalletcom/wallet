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
    #[serde(default)]
    evm_calls: Vec<EvmCallConfig>,
}

impl CacheConfig {
    pub(crate) fn rules(&self, chain: Chain) -> ChainCacheRules {
        let cache = self
            .chain_types
            .get(&chain.chain_type())
            .into_iter()
            .flatten()
            .chain(self.chains.get(&chain).into_iter().flatten())
            .cloned()
            .collect();
        let evm_calls = self
            .evm_calls
            .iter()
            .filter_map(|config| {
                config.contracts.get(&chain).map(|contract| EvmCallRule {
                    contract: contract.clone(),
                    selector: config.selector.clone(),
                    ttl: config.ttl,
                })
            })
            .collect();

        ChainCacheRules { cache, evm_calls }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChainCacheRules {
    pub(crate) cache: Vec<CacheRule>,
    pub(crate) evm_calls: Vec<EvmCallRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CacheRule {
    pub(crate) path: Option<String>,
    pub(crate) method: Option<String>,
    pub(crate) rpc_method: Option<String>,
    #[serde(default, alias = "ttl_seconds", deserialize_with = "duration::deserialize_option")]
    pub(crate) ttl: Option<Duration>,
    #[serde(default)]
    pub(crate) params: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct EvmCallConfig {
    selector: String,
    #[serde(deserialize_with = "duration::deserialize")]
    ttl: Duration,
    contracts: HashMap<Chain, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct EvmCallRule {
    contract: String,
    selector: String,
    pub(crate) ttl: Duration,
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

    pub(crate) fn matches_rpc(&self, rpc_method: &str) -> bool {
        rpc_method != ETH_CALL && self.rpc_method.as_deref() == Some(rpc_method)
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

impl EvmCallRule {
    pub(crate) fn matches(&self, params: &Value) -> bool {
        matches_evm_call(params, &self.contract, &self.selector)
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

    fn evm_call_rule(selector: &str) -> EvmCallRule {
        EvmCallRule {
            contract: CONTRACT.to_string(),
            selector: selector.to_string(),
            ttl: Duration::from_secs(60 * 60),
        }
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

        assert!(rule.matches(&eth_call_params(CONTRACT, "0x1698ee820000")));
        assert!(rule.matches(&eth_call_params(&CONTRACT.to_uppercase().replacen("0X", "0x", 1), "0x1698EE820000")));
        assert!(!rule.matches(&wrong_block));
        assert!(!rule.matches(&eth_call_params("0x2222222222222222222222222222222222222222", "0x1698ee820000")));
        assert!(!rule.matches(&eth_call_params(CONTRACT, "0xaa9d21cb0000")));
        assert!(!rule.matches(&serde_json::json!([
            {
                "to": CONTRACT,
                "data": "0x1698ee82",
                "value": "0x1"
            },
            "latest"
        ])));

        assert!(!evm_call_rule("0x1698").matches(&eth_call_params(CONTRACT, "0x1698ee82")));
    }

    #[test]
    fn test_legacy_rpc_rule_compatibility() {
        let rule: CacheRule = serde_json::from_value(serde_json::json!({
            "rpc_method": "eth_chainId",
            "ttl": "1h"
        }))
        .unwrap();

        assert!(rule.matches_rpc("eth_chainId"));
        assert!(!rule.matches_rpc("eth_blockNumber"));

        let broad_evm_rule: CacheRule = serde_json::from_value(serde_json::json!({
            "rpc_method": "eth_call",
            "ttl": "1h"
        }))
        .unwrap();
        assert!(!broad_evm_rule.matches_rpc(ETH_CALL));
    }

    #[test]
    fn test_evm_call_config_resolves_contract_by_chain() {
        let config: CacheConfig = serde_json::from_value(serde_json::json!({
            "max_memory_mb": 64,
            "evm_calls": [{
                "selector": "0x1698ee82",
                "ttl": "5m",
                "contracts": {
                    "ethereum": CONTRACT
                }
            }]
        }))
        .unwrap();

        let ethereum = config.rules(Chain::Ethereum);
        assert_eq!(ethereum.evm_calls.len(), 1);
        assert_eq!(ethereum.evm_calls[0].ttl, MINUTE * 5);
        assert!(ethereum.evm_calls[0].matches(&eth_call_params(CONTRACT, "0x1698ee82")));
        assert!(config.rules(Chain::Optimism).evm_calls.is_empty());
    }
}
