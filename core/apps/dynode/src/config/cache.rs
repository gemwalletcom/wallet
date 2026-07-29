use std::collections::{HashMap, HashSet};
use std::time::Duration;

use primitives::{Chain, ChainType};
use serde::Deserialize;
use serde_json::Value;
use serde_serializers::{duration, size};

use super::path_without_query;
use crate::cache::decoder::{ContractCall, ContractRequest, ETH_CALL, decode_contract_calls};
use crate::jsonrpc_types::JsonRpcCall;

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
        let contracts = chain_type_config.map(|config| config.contracts.clone()).unwrap_or_default();

        let rules = ChainCacheRules { cache, contracts };
        if rules.is_empty() {
            return None;
        }
        Some(rules)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChainCacheRules {
    cache: Vec<CacheRule>,
    contracts: ContractCacheConfig,
}

impl ChainCacheRules {
    fn is_empty(&self) -> bool {
        self.cache.is_empty() && self.contracts.is_empty()
    }

    pub(crate) fn call_ttl(&self, chain_type: &ChainType, call: &JsonRpcCall) -> Option<Duration> {
        self.contracts.ttl(chain_type, ContractRequest::JsonRpc(call))
    }

    pub(crate) fn request_ttl(&self, chain_type: &ChainType, path: &str, method: &str, body: &[u8]) -> Option<Duration> {
        self.contracts
            .ttl(
                chain_type,
                ContractRequest::Http {
                    path: path_without_query(path),
                    method,
                    body,
                },
            )
            .or_else(|| self.cache.iter().find(|rule| rule.matches_path_request(path, method, Some(body))).and_then(|rule| rule.ttl))
    }

    pub(crate) fn rpc_ttl(&self, method: &str) -> Option<Duration> {
        self.cache.iter().find(|rule| rule.matches_rpc(method)).and_then(|rule| rule.ttl)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChainTypeCacheConfig {
    #[serde(default)]
    rules: Vec<CacheRule>,
    #[serde(default)]
    contracts: ContractCacheConfig,
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

#[derive(Debug, Clone, Default, Deserialize)]
struct ContractCacheConfig {
    #[serde(default)]
    methods: Vec<ContractMethodRule>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContractMethodRule {
    addresses: HashSet<String>,
    identifiers: Vec<String>,
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

impl ContractCacheConfig {
    fn is_empty(&self) -> bool {
        self.methods.iter().all(|method| method.addresses.is_empty() || method.identifiers.is_empty())
    }

    fn ttl(&self, chain_type: &ChainType, request: ContractRequest<'_>) -> Option<Duration> {
        let calls = decode_contract_calls(chain_type, request)?;
        self.ttl_for_calls(chain_type, &calls)
    }

    fn ttl_for_calls(&self, chain_type: &ChainType, calls: &[ContractCall]) -> Option<Duration> {
        self.methods.iter().find_map(|method| method.matches(chain_type, calls).then_some(method.ttl))
    }
}

impl ContractMethodRule {
    fn matches(&self, chain_type: &ChainType, calls: &[ContractCall]) -> bool {
        !calls.is_empty()
            && !self.identifiers.is_empty()
            && calls.len().is_multiple_of(self.identifiers.len())
            && calls.chunks_exact(self.identifiers.len()).all(|chunk| {
                chunk.iter().zip(&self.identifiers).all(|(call, identifier)| {
                    self.addresses.iter().any(|address| address.eq_ignore_ascii_case(&call.address))
                        && match chain_type {
                            ChainType::Ethereum => identifier.eq_ignore_ascii_case(&call.identifier),
                            _ => identifier == &call.identifier,
                        }
                })
            })
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
    fn test_contract_method_sequence_matching() {
        let rule = ContractMethodRule {
            addresses: HashSet::from(["0x25eb".to_string()]),
            identifiers: ["factory::new_pool_key", "factory::pool_simple_info", "factory::pool_id"].map(str::to_string).to_vec(),
            ttl: MINUTE,
        };
        let calls = ["factory::new_pool_key", "factory::pool_simple_info", "factory::pool_id"]
            .map(|identifier| ContractCall {
                address: "0x25eb".to_string(),
                identifier: identifier.to_string(),
            })
            .to_vec();

        assert!(rule.matches(&ChainType::Sui, &calls));
        assert!(rule.matches(&ChainType::Sui, &[calls.clone(), calls].concat()));
        assert!(!rule.matches(
            &ChainType::Sui,
            &[ContractCall {
                address: "0x25eb".to_string(),
                identifier: "pool::calculate_swap_result".to_string(),
            }]
        ));
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
                        "methods": [{
                                "identifiers": ["0x1698ee82"],
                                "ttl": "5m",
                                "addresses": [CONTRACT]
                        }]
                    }
                }
            }
        }))
        .unwrap();

        let ethereum = config.rules(Chain::Ethereum).unwrap();
        assert_eq!(config.max_memory, 64_000_000);
        assert_eq!(ethereum.contracts.methods.len(), 1);
        assert_eq!(
            ethereum.call_ttl(&ChainType::Ethereum, &JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698ee82"))),
            Some(MINUTE * 5)
        );
        assert_eq!(
            config
                .rules(Chain::Optimism)
                .unwrap()
                .call_ttl(&ChainType::Ethereum, &JsonRpcCall::mock_with_params(1, ETH_CALL, eth_call_params(CONTRACT, "0x1698ee82"))),
            Some(MINUTE * 5)
        );
        assert!(config.rules(Chain::Solana).is_none());
    }
}
