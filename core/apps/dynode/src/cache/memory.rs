use crate::config::{CacheConfig, CacheRule, ChainConfig, ChainTypesConfig};
use crate::jsonrpc_types::{JsonRpcCall, JsonRpcRequest, RequestType};
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
    rules: Arc<HashMap<Chain, Vec<CacheRule>>>,
}

impl MemoryCache {
    pub fn new<'a>(config: CacheConfig, chain_types: &ChainTypesConfig, chains: impl IntoIterator<Item = &'a ChainConfig>) -> Self {
        let mut caches = HashMap::new();
        let mut rules = HashMap::new();
        for chain_config in chains {
            let cache_rules = chain_types.cache_rules(chain_config);
            if !cache_rules.is_empty() {
                caches.insert(chain_config.chain, Arc::new(RwLock::new(HashMap::new())));
                rules.insert(chain_config.chain, cache_rules);
            }
        }
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

    fn rule_for_request<'a>(&'a self, chain: &Chain, request_type: &RequestType) -> Option<&'a CacheRule> {
        self.rules.get(chain)?.iter().find(|rule| match request_type {
            RequestType::Regular { path, method, body } => rule.matches_path_request(path, method, Some(body.as_slice())),
            RequestType::JsonRpc(JsonRpcRequest::Single(call)) => rule.matches_rpc(&call.method),
            RequestType::JsonRpc(JsonRpcRequest::Batch(_)) => false,
        })
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
        self.rule_for_request(chain, request_type).and_then(|rule| rule.ttl)
    }

    fn should_cache_call(&self, chain: &Chain, call: &JsonRpcCall) -> Option<Duration> {
        self.rules.get(chain)?.iter().find(|rule| rule.matches_rpc(&call.method)).and_then(|rule| rule.ttl)
    }

    fn should_inflight_request(&self, chain: &Chain, request_type: &RequestType) -> bool {
        matches!(request_type, RequestType::Regular { .. }) && self.rule_for_request(chain, request_type).is_some_and(|rule| rule.inflight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Url;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use reqwest::StatusCode;
    use std::collections::HashMap;

    fn create_test_chain_types() -> ChainTypesConfig {
        serde_json::from_value(serde_json::json!({
            "ethereum": {
                "cache": [
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
            cache: None,
            urls: vec![Url {
                url: "https://example.com".to_string(),
                headers: None,
            }],
        }
    }

    fn create_test_cache() -> MemoryCache {
        let chains = vec![create_chain_config(Chain::Ethereum)];
        MemoryCache::new(CacheConfig { max_memory_mb: 64 }, &create_test_chain_types(), chains.iter())
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

        let response = CachedResponse::new(b"test".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), Duration::from_secs(60));
        cache.set(&chain, "test_key".to_string(), response.clone(), Duration::from_secs(60)).await;

        let cached = cache.get(&chain, "test_key").await.unwrap();
        assert_eq!(cached.body, response.body);
        assert_eq!(cached.status, response.status);
    }

    #[test]
    fn test_should_cache_path_rule() {
        let cache = create_test_cache();
        let chain = Chain::Ethereum;

        let ttl = cache.should_cache_request(&chain, &regular_request("/api/v1/data", "GET", &[]));
        assert_eq!(ttl, Some(Duration::from_secs(300)));

        let ttl = cache.should_cache_request(&chain, &regular_request("/api/v1/data", "POST", &[]));
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_should_cache_with_params() {
        let chain_types: ChainTypesConfig = serde_json::from_value(serde_json::json!({
            "ethereum": {
                "cache": [
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
        let chains = vec![create_chain_config(Chain::Ethereum)];
        let cache = MemoryCache::new(CacheConfig { max_memory_mb: 64 }, &chain_types, chains.iter());
        let chain = Chain::Ethereum;

        let ttl = cache.should_cache_request(&chain, &regular_request("/info", "POST", br#"{"type":"metaAndAssetCtxs"}"#));
        assert_eq!(ttl, Some(Duration::from_secs(200)));

        let ttl = cache.should_cache_request(&chain, &regular_request("/info", "POST", br#"{"type":"other"}"#));
        assert_eq!(ttl, None);

        let ttl = cache.should_cache_request(&chain, &regular_request("/info", "POST", &[]));
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_should_cache_request() {
        let cache = create_test_cache();
        let chain = Chain::Ethereum;

        let request = RequestType::JsonRpc(JsonRpcRequest::Single(JsonRpcCall {
            jsonrpc: "2.0".to_string(),
            method: "eth_blockNumber".to_string(),
            params: serde_json::json!([]),
            id: 1,
        }));

        let ttl = cache.should_cache_request(&chain, &request);
        assert_eq!(ttl, Some(Duration::from_secs(60)));
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
        assert_eq!(ttl, Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_should_cache_with_function_params() {
        let chain_types: ChainTypesConfig = serde_json::from_value(serde_json::json!({
            "aptos": {
                "cache": [
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
        let chains = vec![create_chain_config(Chain::Aptos)];
        let cache = MemoryCache::new(CacheConfig { max_memory_mb: 64 }, &chain_types, chains.iter());
        let chain = Chain::Aptos;

        let body1 = r#"{
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": ["0xdb5247f859ce63dbe8940cf8773be722a60dcc594a8be9aca4b76abceb251b8e"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache_request(&chain, &regular_request("/v1/view", "POST", &body1));
        assert_eq!(ttl, Some(Duration::from_secs(3600)));

        let body2 = r#"{
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": ["0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"]
        }"#
        .as_bytes()
        .to_vec();

        let ttl = cache.should_cache_request(&chain, &regular_request("/v1/view", "POST", &body2));
        assert_eq!(ttl, Some(Duration::from_secs(3600)));

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
        let config = CacheConfig { max_memory_mb: 0 };
        let chains = vec![create_chain_config(Chain::Ethereum)];
        let cache = MemoryCache::new(config, &create_test_chain_types(), chains.iter());
        let chain = Chain::Ethereum;

        let response1 = CachedResponse::new(b"first".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), Duration::from_secs(60));
        cache.set(&chain, "key1".to_string(), response1, Duration::from_secs(60)).await;

        let response2 = CachedResponse::new(b"second".to_vec(), StatusCode::OK.as_u16(), JSON_CONTENT_TYPE.to_string(), Duration::from_secs(60));
        cache.set(&chain, "key2".to_string(), response2, Duration::from_secs(60)).await;

        assert!(cache.get(&chain, "key1").await.is_none());
    }

    #[test]
    fn test_should_inflight_request() {
        let chain_types: ChainTypesConfig = serde_json::from_value(serde_json::json!({
            "tron": {
                "cache": [
                    {
                        "path": "/wallet/getaccount",
                        "method": "POST",
                        "inflight": true
                    }
                ]
            }
        }))
        .unwrap();
        let chains = vec![create_chain_config(Chain::Tron)];
        let cache = MemoryCache::new(CacheConfig { max_memory_mb: 64 }, &chain_types, chains.iter());
        let chain = Chain::Tron;
        let request_type = RequestType::Regular {
            path: "/wallet/getaccount".to_string(),
            method: "POST".to_string(),
            body: br#"{"address":"T...","visible":true}"#.to_vec(),
        };

        assert!(cache.should_inflight_request(&chain, &request_type));
        assert_eq!(cache.should_cache_request(&chain, &request_type), None);
    }

    #[test]
    fn test_chain_cache_replaces_chain_type_cache() {
        let chain_types = create_test_chain_types();
        let mut chain_config = create_chain_config(Chain::Ethereum);
        chain_config.cache = Some(vec![CacheRule {
            path: None,
            method: None,
            rpc_method: Some("eth_getLogs".to_string()),
            ttl: Some(Duration::from_secs(30)),
            inflight: false,
            params: HashMap::new(),
        }]);
        let chains = vec![chain_config];
        let cache = MemoryCache::new(CacheConfig { max_memory_mb: 64 }, &chain_types, chains.iter());

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
            None
        );
        assert_eq!(
            cache.should_cache_call(
                &Chain::Ethereum,
                &JsonRpcCall {
                    jsonrpc: "2.0".to_string(),
                    method: "eth_getLogs".to_string(),
                    params: serde_json::json!([]),
                    id: 1,
                }
            ),
            Some(Duration::from_secs(30))
        );
    }
}
