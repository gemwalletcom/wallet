use crate::config::{CacheConfig, ChainCacheRules, ChainConfig};
use crate::jsonrpc_types::{JsonRpcCall, RequestType};
use crate::proxy::CachedResponse;
use primitives::Chain;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use super::CacheProvider;
use super::types::CacheEntry;

#[derive(Debug, Clone)]
pub struct MemoryCache {
    caches: Arc<HashMap<Chain, Arc<RwLock<HashMap<String, CacheEntry>>>>>,
    max_memory_mb: usize,
    rules: Arc<HashMap<Chain, ChainCacheRules>>,
}

impl MemoryCache {
    pub fn new<'a>(config: CacheConfig, chains: impl IntoIterator<Item = &'a ChainConfig>) -> Self {
        let rules = chains
            .into_iter()
            .filter_map(|chain_config| {
                let cache_rules = config.rules(chain_config.chain);
                (!cache_rules.cache.is_empty() || !cache_rules.evm_calls.is_empty()).then_some((chain_config.chain, cache_rules))
            })
            .collect::<HashMap<_, _>>();
        let caches = rules.keys().copied().map(|chain| (chain, Arc::new(RwLock::new(HashMap::new())))).collect();
        Self {
            caches: Arc::new(caches),
            max_memory_mb: config.max_memory_mb,
            rules: Arc::new(rules),
        }
    }

    fn max_size_per_chain(&self) -> usize {
        let chain_count = self.caches.len().max(1);
        (self.max_memory_mb * 1_000_000) / chain_count
    }

    fn evict_if_needed(cache: &mut HashMap<String, CacheEntry>, max_size: usize) {
        let mut size = 0;
        cache.retain(|_, entry| {
            if entry.is_expired() {
                false
            } else {
                size += entry.size();
                true
            }
        });

        if size <= max_size {
            return;
        }

        let mut valid_entries: Vec<_> = cache.iter().map(|(key, entry)| (key.clone(), entry.created_at)).collect();
        valid_entries.sort_unstable_by_key(|(_, created)| *created);

        for (key, _) in valid_entries {
            if size <= max_size {
                break;
            }
            if let Some(entry) = cache.remove(&key) {
                size -= entry.size();
            }
        }
    }
}

impl CacheProvider for MemoryCache {
    async fn get(&self, chain: &Chain, key: &str) -> Option<CachedResponse> {
        let cache = self.caches.get(chain)?;
        let read_guard = cache.read().await;
        let entry = read_guard.get(key)?;
        if entry.is_expired() {
            drop(read_guard);
            cache.write().await.remove(key);
            return None;
        }
        Some(entry.response.clone())
    }

    async fn set(&self, chain: &Chain, key: String, response: CachedResponse, ttl: Duration) {
        if let Some(cache) = self.caches.get(chain) {
            let entry = CacheEntry::new(response, ttl);
            let mut guard = cache.write().await;
            guard.insert(key, entry);
            Self::evict_if_needed(&mut guard, self.max_size_per_chain());
        }
    }

    fn should_cache_request(&self, chain: &Chain, request_type: &RequestType) -> Option<Duration> {
        let RequestType::Regular { path, method, body } = request_type else {
            return None;
        };
        self.rules
            .get(chain)?
            .cache
            .iter()
            .find(|rule| rule.matches_path_request(path, method, Some(body)))
            .and_then(|rule| rule.ttl)
    }

    fn should_cache_call(&self, chain: &Chain, call: &JsonRpcCall) -> Option<Duration> {
        let rules = self.rules.get(chain)?;
        rules.cache.iter().find(|rule| rule.matches_rpc(&call.method)).and_then(|rule| rule.ttl).or_else(|| {
            if call.method == crate::config::ETH_CALL {
                rules.evm_calls.iter().find(|rule| rule.matches(&call.params)).map(|rule| rule.ttl)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Url;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use primitives::{HOUR, MINUTE};
    use reqwest::StatusCode;
    fn create_test_cache_config() -> CacheConfig {
        serde_json::from_value(serde_json::json!({
            "max_memory_mb": 64,
            "chain_types": {
                "ethereum": [
                    { "path": "/api/v1/data", "method": "GET", "ttl": "5m" },
                    { "rpc_method": "eth_blockNumber", "ttl": "1m" }
                ]
            }
        }))
        .unwrap()
    }

    fn create_chain_config(chain: Chain) -> ChainConfig {
        ChainConfig {
            chain,
            poll_interval_seconds: None,
            overrides: None,
            allowlist: None,
            urls: vec![Url {
                url: "https://example.com".to_string(),
                headers: None,
            }],
        }
    }

    fn create_test_cache() -> MemoryCache {
        let chains = [create_chain_config(Chain::Ethereum)];
        MemoryCache::new(create_test_cache_config(), chains.iter())
    }

    fn regular_request(path: &str, method: &str, body: &[u8]) -> RequestType {
        RequestType::Regular {
            path: path.to_string(),
            method: method.to_string(),
            body: body.to_vec(),
        }
    }

    #[tokio::test]
    async fn test_set_and_get_cache() {
        let cache = create_test_cache();
        let chain = Chain::Ethereum;

        let response = CachedResponse::new(b"test".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string());
        cache.set(&chain, "test_key".to_string(), response.clone(), MINUTE).await;

        let cached = cache.get(&chain, "test_key").await.unwrap();
        assert_eq!(cached.body, response.body);
        assert_eq!(cached.status, response.status);
    }

    #[test]
    fn test_should_cache_path_rule() {
        let cache = create_test_cache();
        let chain = Chain::Ethereum;

        let ttl = cache.should_cache_request(&chain, &regular_request("/api/v1/data", "GET", &[]));
        assert_eq!(ttl, Some(MINUTE * 5));

        let ttl = cache.should_cache_request(&chain, &regular_request("/api/v1/data", "POST", &[]));
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_should_cache_with_params() {
        let config: CacheConfig = serde_json::from_value(serde_json::json!({
            "max_memory_mb": 64,
            "chain_types": {
                "ethereum": [
                    {
                        "path": "/info",
                        "method": "POST",
                        "ttl": "200s",
                        "params": {
                            "type": "metaAndAssetCtxs"
                        }
                    }
                ]
            }
        }))
        .unwrap();
        let chains = [create_chain_config(Chain::Ethereum)];
        let cache = MemoryCache::new(config, chains.iter());
        let chain = Chain::Ethereum;

        let ttl = cache.should_cache_request(&chain, &regular_request("/info", "POST", br#"{"type":"metaAndAssetCtxs"}"#));
        assert_eq!(ttl, Some(Duration::from_secs(200)));

        let ttl = cache.should_cache_request(&chain, &regular_request("/info", "POST", br#"{"type":"other"}"#));
        assert_eq!(ttl, None);

        let ttl = cache.should_cache_request(&chain, &regular_request("/info", "POST", &[]));
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_should_cache_call() {
        let cache = create_test_cache();
        let chain = Chain::Ethereum;

        let call = JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: serde_json::json!([]),
            id: 1,
        };

        let ttl = cache.should_cache_call(&chain, &call);
        assert_eq!(ttl, Some(MINUTE));
    }

    #[test]
    fn test_should_cache_with_function_params() {
        let config: CacheConfig = serde_json::from_value(serde_json::json!({
            "max_memory_mb": 64,
            "chain_types": {
                "aptos": [
                    {
                        "path": "/v1/view",
                        "method": "POST",
                        "ttl": "1h",
                        "params": {
                            "function": "0x1::delegation_pool::operator_commission_percentage"
                        }
                    }
                ]
            }
        }))
        .unwrap();
        let chains = [create_chain_config(Chain::Aptos)];
        let cache = MemoryCache::new(config, chains.iter());
        let chain = Chain::Aptos;

        let body1 = r#"{
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": ["0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache_request(&chain, &regular_request("/v1/view", "POST", &body1));
        assert_eq!(ttl, Some(HOUR));

        let body2 = r#"{
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": ["0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache_request(&chain, &regular_request("/v1/view", "POST", &body2));
        assert_eq!(ttl, Some(HOUR));

        let body3 = r#"{
            "function": "0x1::other_module::other_function",
            "type_arguments": [],
            "arguments": ["0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache_request(&chain, &regular_request("/v1/view", "POST", &body3));
        assert_eq!(ttl, None);
    }

    #[tokio::test]
    async fn test_eviction() {
        let config: CacheConfig = serde_json::from_value(serde_json::json!({ "max_memory_mb": 0 })).unwrap();
        let chains = [create_chain_config(Chain::Ethereum)];
        let cache = MemoryCache::new(config, chains.iter());
        let chain = Chain::Ethereum;

        let response1 = CachedResponse::new(b"first".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string());
        cache.set(&chain, "key1".to_string(), response1, MINUTE).await;

        let response2 = CachedResponse::new(b"second".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string());
        cache.set(&chain, "key2".to_string(), response2, MINUTE).await;

        assert!(cache.get(&chain, "key1").await.is_none());
    }

    #[test]
    fn test_chain_cache_extends_chain_type_cache() {
        const CONTRACT: &str = "0x1111111111111111111111111111111111111111";
        const SELECTOR: &str = "0x1698ee82";

        let config: CacheConfig = serde_json::from_value(serde_json::json!({
            "max_memory_mb": 64,
            "chain_types": {
                "ethereum": [
                    { "rpc_method": "eth_blockNumber", "ttl": "1m" }
                ]
            },
            "evm_calls": [
                {
                    "selector": SELECTOR,
                    "ttl": "30s",
                    "contracts": {
                        "ethereum": CONTRACT
                    }
                }
            ]
        }))
        .unwrap();
        let chains = [create_chain_config(Chain::Ethereum)];
        let cache = MemoryCache::new(config, chains.iter());

        assert_eq!(
            cache.should_cache_call(
                &Chain::Ethereum,
                &JsonRpcCall {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_blockNumber".to_string(),
                    params: serde_json::json!([]),
                    id: 1,
                }
            ),
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            cache.should_cache_call(
                &Chain::Ethereum,
                &JsonRpcCall {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_call".to_string(),
                    params: serde_json::json!([
                        {
                            "to": CONTRACT,
                            "data": SELECTOR
                        },
                        "latest"
                    ]),
                    id: 1,
                }
            ),
            Some(Duration::from_secs(30))
        );
    }
}
