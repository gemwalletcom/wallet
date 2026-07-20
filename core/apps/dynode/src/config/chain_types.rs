use std::collections::HashMap;

use primitives::{Chain, ChainType};
use serde::Deserialize;

use crate::jsonrpc_types::RequestType;

use super::{AllowlistConfig, CacheRule, ChainConfig};

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

        self.chain_type_config(chain_config).is_none_or(|config| config.allows(chain_config.chain, request_type))
    }

    pub fn cache_rules(&self, chain_config: &ChainConfig) -> Vec<CacheRule> {
        if let Some(rules) = chain_config.cache.as_ref() {
            return rules.clone();
        }

        self.chain_type_config(chain_config).map(|config| config.cache(chain_config.chain)).unwrap_or_default()
    }

    fn chain_type_config(&self, chain_config: &ChainConfig) -> Option<&ChainTypeConfig> {
        self.chain_types.get(&chain_config.chain.chain_type())
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

    fn cache(&self, chain: Chain) -> Vec<CacheRule> {
        let mut rules = self.policy.cache.clone().unwrap_or_default();
        if let Some(chain_rules) = self.chains.get(&chain).and_then(|config| config.cache.as_ref()) {
            rules.extend(chain_rules.clone());
        }
        rules
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChainPolicyConfig {
    allowlist: Option<AllowlistConfig>,
    cache: Option<Vec<CacheRule>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Url;
    use primitives::Chain;
    use serde_json::json;

    fn jsonrpc(method: &str) -> RequestType {
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

    fn chain_config(chain: Chain) -> ChainConfig {
        ChainConfig {
            chain,
            poll_interval_seconds: None,
            overrides: None,
            allowlist: None,
            cache: None,
            urls: vec![Url {
                url: "https://example.com".to_string(),
                headers: None,
            }],
        }
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

        assert!(config.allows(&chain_config(Chain::Ethereum), &jsonrpc("eth_call")));
        assert!(config.allows(&chain_config(Chain::Arbitrum), &jsonrpc("eth_call")));
        assert!(!config.allows(&chain_config(Chain::Ethereum), &jsonrpc("trace_replayTransaction")));
    }

    #[test]
    fn test_unconfigured_and_empty_allowlist_are_unrestricted() {
        let config: ChainTypesConfig = serde_json::from_value(json!({
            "solana": {}
        }))
        .unwrap();

        assert!(config.allows(&chain_config(Chain::Tron), &jsonrpc("unknown_method")));
        assert!(config.allows(&chain_config(Chain::Solana), &jsonrpc("unknown_method")));
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

        assert!(config.allows(&chain_config(Chain::Thorchain), &balance));
        assert!(config.allows(&chain_config(Chain::Thorchain), &quote));
        assert!(!config.allows(&chain_config(Chain::Thorchain), &denied));
    }

    #[test]
    fn test_chain_cache_extends_chain_type_cache() {
        let config: ChainTypesConfig = serde_json::from_value(json!({
            "cosmos": {
                "cache": [
                    { "path": "/cosmos/staking/v1beta1/validators", "method": "GET", "ttl": "1d" }
                ],
                "chains": {
                    "thorchain": {
                        "cache": [
                            { "path": "/thorchain/inbound_addresses", "method": "GET", "ttl": "10m" }
                        ]
                    }
                }
            }
        }))
        .unwrap();

        let rules = config.cache_rules(&chain_config(Chain::Thorchain));
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].path.as_deref(), Some("/cosmos/staking/v1beta1/validators"));
        assert_eq!(rules[1].path.as_deref(), Some("/thorchain/inbound_addresses"));
    }
}
