use std::collections::HashMap;

use primitives::{Chain, ChainType};
use serde::Deserialize;

use crate::jsonrpc_types::RequestType;

use super::cache::ContractCacheConfig;
use super::{AllowlistConfig, CacheRule, CacheRules, ChainConfig};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
pub struct ChainTypesConfig {
    chain_types: HashMap<ChainType, ChainTypeConfig>,
}

impl ChainTypesConfig {
    pub fn allows(&self, chain_config: &ChainConfig, request_type: &RequestType) -> bool {
        if let Some(allowlist) = chain_config.allowlist.as_ref() {
            return allowlist.allows(request_type);
        }

        self.chain_types
            .get(&chain_config.chain.chain_type())
            .is_none_or(|config| config.allows(chain_config.chain, request_type))
    }

    pub(crate) fn cache_rules(&self, chain: Chain) -> Option<CacheRules> {
        let config = self.chain_types.get(&chain.chain_type())?;
        let policy = config.chains.get(&chain);
        let cache = config.policy.cache.iter().chain(policy.into_iter().flat_map(|policy| &policy.cache)).cloned().collect();
        let contracts = ContractCacheConfig {
            methods: config
                .policy
                .contracts
                .methods
                .iter()
                .chain(policy.into_iter().flat_map(|policy| &policy.contracts.methods))
                .cloned()
                .collect(),
        };
        let rules = CacheRules { cache, contracts };
        (!rules.is_empty()).then_some(rules)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChainTypeConfig {
    #[serde(flatten)]
    policy: ChainPolicyConfig,
    #[serde(default)]
    chains: HashMap<Chain, ChainPolicyConfig>,
}

impl ChainTypeConfig {
    fn allows(&self, chain: Chain, request_type: &RequestType) -> bool {
        let allowlists = [self.policy.allowlist.as_ref(), self.chains.get(&chain).and_then(|config| config.allowlist.as_ref())];
        let mut has_rules = false;

        for allowlist in allowlists.into_iter().flatten() {
            if allowlist.is_empty() {
                continue;
            }
            has_rules = true;
            if allowlist.allows(request_type) {
                return true;
            }
        }

        !has_rules
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChainPolicyConfig {
    allowlist: Option<AllowlistConfig>,
    #[serde(default)]
    cache: Vec<CacheRule>,
    #[serde(default)]
    contracts: ContractCacheConfig,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::testkit::config::{chain_config, jsonrpc};
    use primitives::Chain;
    use serde_json::json;

    #[test]
    fn test_cache_policies_extend_without_changing_access() {
        let config: ChainTypesConfig = serde_json::from_value(json!({
            "ethereum": {
                "allowlist": [{ "rpc_method": "eth_chainId" }],
                "cache": [{ "rpc_method": "eth_blockNumber", "ttl": "1m" }],
                "chains": {
                    "ethereum": {
                        "cache": [
                            { "rpc_method": "eth_blockNumber", "ttl": "2m" },
                            { "rpc_method": "eth_chainId", "ttl": "3m" }
                        ]
                    }
                }
            }
        }))
        .unwrap();
        let ethereum = config.cache_rules(Chain::Ethereum).unwrap();
        let optimism = config.cache_rules(Chain::Optimism).unwrap();
        assert_eq!(ethereum.rpc_ttl("eth_blockNumber"), Some(Duration::from_secs(60)));
        assert_eq!(ethereum.rpc_ttl("eth_chainId"), Some(Duration::from_secs(180)));
        assert_eq!(optimism.rpc_ttl("eth_blockNumber"), Some(Duration::from_secs(60)));
        assert_eq!(optimism.rpc_ttl("eth_chainId"), None);
        let chain = chain_config(Chain::Ethereum, "https://example.com");
        assert!(config.allows(&chain, &jsonrpc("eth_chainId")));
        assert!(!config.allows(&chain, &jsonrpc("eth_blockNumber")));
        assert!(config.cache_rules(Chain::Solana).is_none());
    }

    #[test]
    fn test_allows_chain_type_policy() {
        let config: ChainTypesConfig = serde_json::from_value(json!({
            "ethereum": {
                "allowlist": [
                    { "rpc_method": "eth_call" }
                ]
            }
        }))
        .unwrap();

        assert!(config.allows(&chain_config(Chain::Ethereum, "https://example.com"), &jsonrpc("eth_call")));
        assert!(config.allows(&chain_config(Chain::Arbitrum, "https://example.com"), &jsonrpc("eth_call")));
        assert!(!config.allows(&chain_config(Chain::Ethereum, "https://example.com"), &jsonrpc("unsupported_method")));
    }

    #[test]
    fn test_unconfigured_and_empty_allowlist_are_unrestricted() {
        let config: ChainTypesConfig = serde_json::from_value(json!({
            "solana": {}
        }))
        .unwrap();

        assert!(config.allows(&chain_config(Chain::Tron, "https://example.com"), &jsonrpc("unknown_method")));
        assert!(config.allows(&chain_config(Chain::Solana, "https://example.com"), &jsonrpc("unknown_method")));
    }

    #[test]
    fn test_chain_allowlist_extends_chain_type_allowlist() {
        let config: ChainTypesConfig = serde_json::from_value(json!({
            "cosmos": {
                "allowlist": [
                    { "path": "/cosmos/bank/v1beta1/balances/**", "method": "GET" }
                ],
                "chains": {
                    "thorchain": {
                        "allowlist": [
                            { "path": "/thorchain/quote/swap", "method": "GET" }
                        ]
                    }
                }
            }
        }))
        .unwrap();
        let balance = RequestType::from_request("GET", "/cosmos/bank/v1beta1/balances/thor15r90lnu7wa4ll0ex6rqu77ysavfjkehazqse5u".to_string(), Vec::new());
        let quote = RequestType::from_request("GET", "/thorchain/quote/swap?from_asset=SOL.SOL".to_string(), Vec::new());
        let denied = RequestType::from_request("GET", "/thorchain/vaults/asgard".to_string(), Vec::new());

        assert!(config.allows(&chain_config(Chain::Thorchain, "https://example.com"), &balance));
        assert!(config.allows(&chain_config(Chain::Thorchain, "https://example.com"), &quote));
        assert!(!config.allows(&chain_config(Chain::Thorchain, "https://example.com"), &denied));
    }
}
