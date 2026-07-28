use std::collections::{HashMap, HashSet};
use std::time::Duration;

use primitives::{Chain, ChainType};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::{duration, size};

use super::path_without_query;
use crate::jsonrpc_types::JsonRpcCall;

pub(crate) const ETH_CALL: &str = "eth_call";

#[derive(Debug, Default, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(deserialize_with = "size::deserialize")]
    pub max_memory: usize,
    #[serde(default)]
    chain_types: HashMap<ChainType, ChainTypeCacheConfig>,
    #[serde(default)]
    chains: HashMap<Chain, Vec<CacheRule>>,
}

impl CacheConfig {
    pub(crate) fn rules(&self, chain: Chain) -> Option<ChainCacheRules> {
        let chain_type = chain.chain_type();
        let chain_type_config = self.chain_types.get(&chain_type);
        let cache = chain_type_config
            .into_iter()
            .flat_map(|config| &config.rules)
            .chain(self.chains.get(&chain).into_iter().flatten())
            .cloned()
            .collect();
        let contracts = chain_type_config.and_then(|config| config.contracts.clone()).filter(|contracts| !contracts.is_empty());

        let rules = ChainCacheRules { cache, contracts };
        if rules.is_empty() {
            return None;
        }
        Some(rules)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChainCacheRules {
    pub(crate) cache: Vec<CacheRule>,
    contracts: Option<Contracts>,
}

impl ChainCacheRules {
    fn is_empty(&self) -> bool {
        self.cache.is_empty() && self.contracts.is_none()
    }

    pub(crate) fn ttl(&self, call: &JsonRpcCall) -> Option<Duration> {
        self.contracts.as_ref()?.ttl(call)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChainTypeCacheConfig {
    #[serde(default)]
    rules: Vec<CacheRule>,
    contracts: Option<Contracts>,
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
struct Contracts {
    addresses: HashSet<String>,
    methods: HashMap<String, Method>,
}

#[derive(Debug, Clone, Deserialize)]
struct Method {
    #[serde(deserialize_with = "duration::deserialize")]
    ttl: Duration,
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

impl Contracts {
    fn is_empty(&self) -> bool {
        self.addresses.is_empty() || self.methods.is_empty()
    }

    fn ttl(&self, call: &JsonRpcCall) -> Option<Duration> {
        if call.method != ETH_CALL {
            return None;
        }

        let params = call.params.as_array()?;
        if params.len() != 2 || params.get(1).and_then(Value::as_str) != Some("latest") {
            return None;
        }

        let call = params.first()?.as_object()?;
        if call.len() != 2 {
            return None;
        }

        let address = call.get("to")?.as_str()?;
        if !self.addresses.contains(address) {
            return None;
        }

        let method = call.get("data")?.as_str()?.get(..10)?;
        self.methods.get(method).map(|method| method.ttl)
    }
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
    fn test_contract_call_matching() {
        let rules = Contracts {
            addresses: HashSet::from([CONTRACT.to_string()]),
            methods: HashMap::from([("0x1698ee82".to_string(), Method { ttl: MINUTE })]),
        };
        let wrong_block = serde_json::json!([{ "to": CONTRACT, "data": "0x1698ee820000" }, "pending"]);

        assert_eq!(
            rules.ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698ee820000"))),
            Some(MINUTE)
        );
        assert_eq!(rules.ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698EE820000"))), None);
        assert_eq!(rules.ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, wrong_block)), None);
        assert_eq!(
            rules.ttl(&JsonRpcCall::mock_with_params(
                1,
                ETH_CALL,
                eth_call_params("0x2222222222222222222222222222222222222222", "0x1698ee820000")
            )),
            None
        );
        assert_eq!(rules.ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0xaa9d21cb0000"))), None);
        assert_eq!(
            rules.ttl(&JsonRpcCall::mock_with_params(
                1,
                ETH_CALL,
                serde_json::json!([
                    {
                        "to": CONTRACT,
                        "data": "0x1698ee82",
                        "value": "0x1"
                    },
                    "latest"
                ])
            )),
            None
        );
        assert_eq!(rules.ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698"))), None);
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
    fn test_contract_call_config_resolves_by_chain_type() {
        let config: CacheConfig = serde_json::from_value(serde_json::json!({
            "max_memory": "64 MB",
            "chain_types": {
                "ethereum": {
                    "contracts": {
                        "addresses": [CONTRACT],
                        "methods": {
                            "0x1698ee82": {
                                "ttl": "5m"
                            }
                        }
                    }
                }
            }
        }))
        .unwrap();

        let ethereum = config.rules(Chain::Ethereum).unwrap();
        assert_eq!(config.max_memory, 64_000_000);
        assert_eq!(ethereum.contracts.as_ref().unwrap().addresses.len(), 1);
        assert_eq!(ethereum.contracts.as_ref().unwrap().methods.len(), 1);
        assert_eq!(
            ethereum.ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698ee82"))),
            Some(MINUTE * 5)
        );
        assert_eq!(
            config
                .rules(Chain::Optimism)
                .unwrap()
                .ttl(&JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698ee82"))),
            Some(MINUTE * 5)
        );
        assert!(config.rules(Chain::Solana).is_none());
    }
}
