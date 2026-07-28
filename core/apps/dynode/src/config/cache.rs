use std::collections::{HashMap, HashSet};
use std::time::Duration;

use primitives::{Chain, ChainType};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::{duration, size};

use super::path_without_query;

pub(crate) const ETH_CALL: &str = "eth_call";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(deserialize_with = "size::deserialize")]
    pub max_memory: usize,
    #[serde(default)]
    chain_types: HashMap<ChainType, Vec<CacheRule>>,
    #[serde(default)]
    chains: HashMap<Chain, Vec<CacheRule>>,
    #[serde(default)]
    evm_calls: HashMap<ChainType, EvmCallConfig>,
}

impl CacheConfig {
    pub(crate) fn rules(&self, chain: Chain) -> Option<ChainCacheRules> {
        let cache = self
            .chain_types
            .get(&chain.chain_type())
            .into_iter()
            .flatten()
            .chain(self.chains.get(&chain).into_iter().flatten())
            .cloned()
            .collect();
        let evm_calls = self.evm_calls.get(&chain.chain_type()).map(EvmCallRules::from).unwrap_or_default();

        let rules = ChainCacheRules { cache, evm_calls };
        if rules.is_empty() {
            return None;
        }
        Some(rules)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChainCacheRules {
    pub(crate) cache: Vec<CacheRule>,
    evm_calls: EvmCallRules,
}

impl ChainCacheRules {
    fn is_empty(&self) -> bool {
        self.cache.is_empty() && self.evm_calls.is_empty()
    }

    pub(crate) fn evm_call_ttl(&self, params: &Value) -> Option<Duration> {
        self.evm_calls.ttl_for_params(params)
    }
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
    contracts: HashSet<String>,
    selectors: HashMap<String, EvmSelectorConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct EvmSelectorConfig {
    #[serde(deserialize_with = "duration::deserialize")]
    ttl: Duration,
}

#[derive(Debug, Default, Clone)]
struct EvmCallRules {
    contracts: HashSet<[u8; 20]>,
    selectors: HashMap<[u8; 4], Duration>,
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

impl From<&EvmCallConfig> for EvmCallRules {
    fn from(config: &EvmCallConfig) -> Self {
        Self {
            contracts: config.contracts.iter().filter_map(|contract| decode_hex(contract)).collect(),
            selectors: config
                .selectors
                .iter()
                .filter_map(|(selector, config)| decode_hex(selector).map(|selector| (selector, config.ttl)))
                .collect(),
        }
    }
}

impl EvmCallRules {
    fn is_empty(&self) -> bool {
        self.contracts.is_empty() || self.selectors.is_empty()
    }

    fn ttl_for_params(&self, params: &Value) -> Option<Duration> {
        let params = params.as_array()?;
        if params.len() != 2 || params.get(1).and_then(Value::as_str) != Some("latest") {
            return None;
        }

        let call = params.first()?.as_object()?;
        if call.len() != 2 {
            return None;
        }

        let contract = decode_hex(call.get("to")?.as_str()?)?;
        if !self.contracts.contains(&contract) {
            return None;
        }

        let selector = decode_hex(call.get("data")?.as_str()?.get(..10)?)?;
        self.selectors.get(&selector).copied()
    }
}

fn decode_hex<const N: usize>(value: &str) -> Option<[u8; N]> {
    let mut bytes = [0; N];
    hex::decode_to_slice(value.strip_prefix("0x")?, &mut bytes).ok()?;
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::MINUTE;

    const CONTRACT: &str = "0x1111111111111111111111111111111111111111";

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
        let rules = EvmCallRules {
            contracts: HashSet::from([decode_hex(CONTRACT).unwrap()]),
            selectors: HashMap::from([(decode_hex("0x1698ee82").unwrap(), MINUTE)]),
        };
        let wrong_block = serde_json::json!([{ "to": CONTRACT, "data": "0x1698ee820000" }, "pending"]);

        assert_eq!(rules.ttl_for_params(&eth_call_params(CONTRACT, "0x1698ee820000")), Some(MINUTE));
        assert_eq!(
            rules.ttl_for_params(&eth_call_params(&CONTRACT.to_uppercase().replacen("0X", "0x", 1), "0x1698EE820000")),
            Some(MINUTE)
        );
        assert_eq!(rules.ttl_for_params(&wrong_block), None);
        assert_eq!(rules.ttl_for_params(&eth_call_params("0x2222222222222222222222222222222222222222", "0x1698ee820000")), None);
        assert_eq!(rules.ttl_for_params(&eth_call_params(CONTRACT, "0xaa9d21cb0000")), None);
        assert_eq!(
            rules.ttl_for_params(&serde_json::json!([
                {
                    "to": CONTRACT,
                    "data": "0x1698ee82",
                    "value": "0x1"
                },
                "latest"
            ])),
            None
        );
        assert_eq!(decode_hex::<4>("0x1698"), None);
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
    fn test_evm_call_config_resolves_contract_by_chain_type() {
        let uppercase_contract = CONTRACT.to_uppercase().replacen("0X", "0x", 1);
        let config: CacheConfig = serde_json::from_value(serde_json::json!({
            "max_memory": "64 MB",
            "evm_calls": {
                "ethereum": {
                    "contracts": [CONTRACT, uppercase_contract],
                    "selectors": {
                        "0x1698ee82": {
                            "ttl": "5m"
                        }
                    }
                }
            }
        }))
        .unwrap();

        let ethereum = config.rules(Chain::Ethereum).unwrap();
        assert_eq!(config.max_memory, 64_000_000);
        assert_eq!(ethereum.evm_calls.contracts.len(), 1);
        assert_eq!(ethereum.evm_calls.selectors.len(), 1);
        assert_eq!(ethereum.evm_call_ttl(&eth_call_params(CONTRACT, "0x1698ee82")), Some(MINUTE * 5));
        assert_eq!(
            config.rules(Chain::Optimism).unwrap().evm_call_ttl(&eth_call_params(CONTRACT, "0x1698ee82")),
            Some(MINUTE * 5)
        );
        assert!(config.rules(Chain::Solana).is_none());
    }
}
