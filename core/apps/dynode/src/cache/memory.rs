use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use primitives::Chain;
use tokio::sync::RwLock;

use super::types::CacheEntry;
use crate::config::routes::RouteConfig;
use crate::config::{CacheConfig, CacheRules, ChainConfig, ChainTypesConfig};
use crate::jsonrpc_types::{JsonRpcCall, RequestType};
use crate::proxy::ProxyResponse;

#[derive(Debug, PartialEq, Eq, Hash)]
enum CacheScope {
    Chain(Chain),
    Provider { group: String, service: String },
}

impl CacheScope {
    fn provider(group: &str, service: &str) -> Self {
        Self::Provider {
            group: group.to_string(),
            service: service.to_string(),
        }
    }
}

#[derive(Debug)]
struct NamespaceCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    rules: CacheRules,
}

impl NamespaceCache {
    fn new(rules: CacheRules) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            rules,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RequestCache {
    namespaces: Arc<HashMap<CacheScope, NamespaceCache>>,
    max_memory: usize,
}

impl RequestCache {
    pub(crate) fn for_chains<'a>(config: &CacheConfig, chain_types: &ChainTypesConfig, chains: impl IntoIterator<Item = &'a ChainConfig>) -> Self {
        let rules = chains
            .into_iter()
            .filter_map(|chain| chain_types.cache_rules(chain.chain).map(|rules| (CacheScope::Chain(chain.chain), rules)));
        Self::new(config.memory.max, rules)
    }

    pub(crate) fn for_routes(config: &CacheConfig, routes: &HashMap<String, RouteConfig>) -> Self {
        let rules = routes
            .iter()
            .filter(|(_, route)| !route.cache.is_empty())
            .map(|(service, route)| (CacheScope::provider(&route.group, service), CacheRules::from_rules(route.cache.clone())));
        Self::new(config.memory.max, rules)
    }

    fn new(max_memory: usize, rules: impl IntoIterator<Item = (CacheScope, CacheRules)>) -> Self {
        Self {
            namespaces: Arc::new(rules.into_iter().map(|(scope, rules)| (scope, NamespaceCache::new(rules))).collect()),
            max_memory,
        }
    }

    pub(crate) fn provider_ttl(&self, group: &str, service: &str, path: &str, method: &str, body: &[u8]) -> Option<Duration> {
        self.namespaces.get(&CacheScope::provider(group, service))?.rules.path_ttl(path, method, body)
    }

    pub(crate) async fn get_provider(&self, group: &str, service: &str, key: &str) -> Option<ProxyResponse> {
        self.get_scoped(&CacheScope::provider(group, service), key).await
    }

    pub(crate) async fn set_provider(&self, group: &str, service: &str, key: String, response: ProxyResponse, ttl: Duration) {
        self.set_scoped(&CacheScope::provider(group, service), key, response, ttl).await;
    }

    async fn get_scoped(&self, scope: &CacheScope, key: &str) -> Option<ProxyResponse> {
        let cache = self.namespaces.get(scope)?;
        let read_guard = cache.entries.read().await;
        let entry = read_guard.get(key)?;
        if entry.is_expired() {
            drop(read_guard);
            let mut write_guard = cache.entries.write().await;
            if write_guard.get(key).is_some_and(CacheEntry::is_expired) {
                write_guard.remove(key);
            }
            return None;
        }
        Some(entry.response.clone().into_cached())
    }

    async fn set_scoped(&self, scope: &CacheScope, key: String, response: ProxyResponse, ttl: Duration) {
        if let Some(cache) = self.namespaces.get(scope) {
            let entry = CacheEntry::new(response, ttl);
            let mut guard = cache.entries.write().await;
            guard.insert(key, entry);
            Self::evict_if_needed(&mut guard, self.max_memory / self.namespaces.len());
        }
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

    pub(crate) async fn get(&self, chain: &Chain, key: &str) -> Option<ProxyResponse> {
        self.get_scoped(&CacheScope::Chain(*chain), key).await
    }

    pub(crate) async fn set(&self, chain: &Chain, key: String, response: ProxyResponse, ttl: Duration) {
        self.set_scoped(&CacheScope::Chain(*chain), key, response, ttl).await;
    }

    pub(crate) fn should_cache_request(&self, chain: &Chain, request_type: &RequestType) -> Option<Duration> {
        let RequestType::Regular { path, method, body } = request_type else {
            return None;
        };
        self.namespaces.get(&CacheScope::Chain(*chain))?.rules.request_ttl(&chain.chain_type(), path, method, body)
    }

    pub(crate) fn should_cache_call(&self, chain: &Chain, call: &JsonRpcCall) -> Option<Duration> {
        let rules = &self.namespaces.get(&CacheScope::Chain(*chain))?.rules;
        rules.call_ttl(&chain.chain_type(), call).or_else(|| rules.rpc_ttl(&call.method))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use primitives::{HOUR, MINUTE};
    use reqwest::StatusCode;
    use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

    use super::*;
    use crate::config::MemoryConfig;
    use crate::proxy::CacheStatus;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use crate::testkit::config::chain_config;
    fn create_test_cache_config() -> CacheConfig {
        CacheConfig {
            memory: MemoryConfig { max: 64_000_000 },
        }
    }

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

    fn create_test_cache() -> RequestCache {
        let chains = [chain_config(Chain::Ethereum, "https://example.com")];
        RequestCache::for_chains(&create_test_cache_config(), &create_test_chain_types(), chains.iter())
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

        let response = ProxyResponse::with_content_type(StatusCode::OK.as_u16(), b"test".to_vec(), JSON_CONTENT_TYPE);
        cache.set(&chain, "test_key".to_string(), response.clone(), MINUTE).await;

        let cached = cache.get(&chain, "test_key").await.unwrap();
        assert_eq!(cached, response.clone().into_cached());
        for (request_id, latency, expected_latency) in [("first", 5, "5ms"), ("second", 10, "10ms")] {
            let annotated = cache
                .get(&chain, "test_key")
                .await
                .unwrap()
                .with_proxy_headers(request_id, Duration::from_millis(latency), CacheStatus::Hit);
            assert_eq!(
                annotated.headers,
                HeaderMap::from_iter([
                    (CONTENT_TYPE, HeaderValue::from_static(JSON_CONTENT_TYPE)),
                    ("x-request-id".parse().unwrap(), HeaderValue::from_static(request_id)),
                    ("x-upstream-latency".parse().unwrap(), HeaderValue::from_static(expected_latency)),
                    ("x-cache".parse().unwrap(), HeaderValue::from_static("HIT")),
                ])
            );
        }
        assert_eq!(cache.get(&chain, "test_key").await, Some(response.into_cached()));
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
        let config: ChainTypesConfig = serde_json::from_value(serde_json::json!({
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
        let chains = [chain_config(Chain::Ethereum, "https://example.com")];
        let cache = RequestCache::for_chains(&create_test_cache_config(), &config, chains.iter());
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

        let call = JsonRpcCall::mock(1, "eth_blockNumber");

        let ttl = cache.should_cache_call(&chain, &call);
        assert_eq!(ttl, Some(MINUTE));
    }

    #[test]
    fn test_should_cache_with_function_params() {
        let config: ChainTypesConfig = serde_json::from_value(serde_json::json!({
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
        let chains = [chain_config(Chain::Aptos, "https://example.com")];
        let cache = RequestCache::for_chains(&create_test_cache_config(), &config, chains.iter());
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

    fn provider_routes() -> HashMap<String, RouteConfig> {
        let enabled: RouteConfig = serde_json::from_value(serde_json::json!({
            "group": "evm", "selection": "ordered", "endpoints": [],
            "cache": [{ "path": "/info", "method": "POST", "params": { "type": "meta" }, "ttl": "1m" }]
        }))
        .unwrap();
        let disabled: RouteConfig = serde_json::from_value(serde_json::json!({ "group": "evm", "selection": "ordered", "endpoints": [] })).unwrap();
        HashMap::from([("ethereum".into(), enabled), ("disabled".into(), disabled)])
    }

    #[tokio::test]
    async fn test_cache_namespaces_and_provider_headers() {
        let nodes = create_test_cache();
        let cache = RequestCache::for_routes(&create_test_cache_config(), &provider_routes());
        let node = ProxyResponse::with_content_type(200, b"node".to_vec(), JSON_CONTENT_TYPE);
        let mut headers = HeaderMap::from_iter([(CONTENT_TYPE, HeaderValue::from_static("application/json"))]);
        headers.append("x-provider", HeaderValue::from_static("first"));
        headers.append("x-provider", HeaderValue::from_static("second"));
        let provider = ProxyResponse::new(200, headers.clone(), b"provider".to_vec());
        nodes.set(&Chain::Ethereum, "same".into(), node.clone(), MINUTE).await;
        cache.set_provider("evm", "ethereum", "same".into(), provider.clone(), MINUTE).await;
        cache.set_provider("evm", "disabled", "same".into(), provider.clone(), MINUTE).await;

        assert_eq!(nodes.get(&Chain::Ethereum, "same").await, Some(node.into_cached()));
        let cached = cache.get_provider("evm", "ethereum", "same").await.unwrap();
        assert_eq!(
            (cached.status, &cached.headers, cached.body.as_slice(), cached.is_from_cache()),
            (200, &headers, b"provider".as_slice(), true)
        );
        assert_eq!(cache.get_provider("other", "ethereum", "same").await, None);
        assert_eq!(cache.get_provider("evm", "disabled", "same").await, None);
        let mut isolated = cache.get_provider("evm", "ethereum", "same").await.unwrap();
        isolated.headers.clear();
        isolated.body.clear();
        assert_eq!(cache.get_provider("evm", "ethereum", "same").await, Some(provider.into_cached()));
    }

    #[test]
    fn test_provider_cache_requires_explicit_matching_rules() {
        let cache = RequestCache::for_routes(&create_test_cache_config(), &provider_routes());
        for (service, path, method, body, ttl) in [
            ("ethereum", "/info?key=one", "POST", br#"{"type":"meta"}"#.as_slice(), Some(MINUTE)),
            ("ethereum", "/info", "GET", br#"{"type":"meta"}"#.as_slice(), None),
            ("ethereum", "/info", "POST", br#"{"type":"other"}"#.as_slice(), None),
            ("ethereum", "/other", "POST", br#"{"type":"meta"}"#.as_slice(), None),
            ("disabled", "/info", "POST", br#"{"type":"meta"}"#.as_slice(), None),
        ] {
            assert_eq!(cache.provider_ttl("evm", service, path, method, body), ttl);
        }
    }

    #[tokio::test]
    async fn test_chain_and_provider_caches_have_independent_budgets() {
        let response = ProxyResponse::with_content_type(200, b"response".to_vec(), JSON_CONTENT_TYPE);
        let size = CacheEntry::new(response.clone(), MINUTE).size();
        let node_config = CacheConfig {
            memory: MemoryConfig { max: 2 * size },
        };
        let provider_config = CacheConfig {
            memory: MemoryConfig { max: 3 * size },
        };
        let chains = [
            chain_config(Chain::Ethereum, "https://example.com"),
            chain_config(Chain::Optimism, "https://optimism.example.com"),
        ];
        let nodes = RequestCache::for_chains(&node_config, &create_test_chain_types(), chains.iter());
        let providers = RequestCache::for_routes(&provider_config, &provider_routes());
        nodes.set(&Chain::Ethereum, "first".into(), response.clone(), MINUTE).await;
        let namespace = nodes.namespaces.get(&CacheScope::Chain(Chain::Ethereum)).unwrap();
        namespace.entries.write().await.get_mut("first").unwrap().created_at = Instant::now() - MINUTE;
        nodes.set(&Chain::Ethereum, "second".into(), response.clone(), MINUTE).await;
        nodes.set(&Chain::Optimism, "other".into(), response.clone(), MINUTE).await;
        for key in ["first", "second", "third"] {
            providers.clone().set_provider("evm", "ethereum", key.into(), response.clone(), MINUTE).await;
        }

        assert_eq!(nodes.get(&Chain::Ethereum, "first").await, None);
        assert_eq!(nodes.get(&Chain::Ethereum, "second").await, Some(response.clone().into_cached()));
        assert_eq!(nodes.get(&Chain::Optimism, "other").await, Some(response.clone().into_cached()));
        for key in ["first", "second", "third"] {
            assert_eq!(providers.get_provider("evm", "ethereum", key).await, Some(response.clone().into_cached()));
        }
        assert_eq!(nodes.get_provider("evm", "ethereum", "first").await, None);
        assert_eq!(providers.get(&Chain::Ethereum, "second").await, None);
        for (cache, expected_size) in [(&nodes, node_config.memory.max), (&providers, provider_config.memory.max)] {
            let mut total = 0;
            for namespace in cache.namespaces.values() {
                total += namespace.entries.read().await.values().map(CacheEntry::size).sum::<usize>();
            }
            assert_eq!(total, expected_size);
        }
    }

    #[tokio::test]
    async fn test_expired_entries_are_removed_in_both_caches() {
        let nodes = create_test_cache();
        let providers = RequestCache::for_routes(&create_test_cache_config(), &provider_routes());
        let response = ProxyResponse::with_content_type(200, b"expired".to_vec(), JSON_CONTENT_TYPE);
        nodes.set(&Chain::Ethereum, "expired".into(), response.clone(), MINUTE).await;
        providers.set_provider("evm", "ethereum", "expired".into(), response, MINUTE).await;
        for cache in [&nodes, &providers] {
            for namespace in cache.namespaces.values() {
                namespace.entries.write().await.get_mut("expired").unwrap().expires_at = Some(Instant::now() - MINUTE);
            }
        }
        assert_eq!(nodes.get(&Chain::Ethereum, "expired").await, None);
        assert_eq!(providers.get_provider("evm", "ethereum", "expired").await, None);
        for cache in [&nodes, &providers] {
            for namespace in cache.namespaces.values() {
                assert_eq!(namespace.entries.read().await.len(), 0);
            }
        }
    }

    #[test]
    fn test_chain_cache_extends_chain_type_cache() {
        const CONTRACT: &str = "0x1111111111111111111111111111111111111111";
        const SELECTOR: &str = "0x1698ee82";

        let config: ChainTypesConfig = serde_json::from_value(serde_json::json!({
                "ethereum": {
                    "cache": [
                        { "rpc_method": "eth_blockNumber", "ttl": "1m" }
                    ],
                    "contracts": {
                        "methods": [{
                            "addresses": [CONTRACT],
                            "identifiers": [SELECTOR],
                            "ttl": "30s"
                        }]
                    }
                }
        }))
        .unwrap();
        let chains = [chain_config(Chain::Ethereum, "https://example.com")];
        let cache = RequestCache::for_chains(&create_test_cache_config(), &config, chains.iter());

        assert_eq!(
            cache.should_cache_call(&Chain::Ethereum, &JsonRpcCall::mock(1, "eth_blockNumber")),
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            cache.should_cache_call(
                &Chain::Ethereum,
                &JsonRpcCall::mock_with_params(
                    1,
                    "eth_call",
                    serde_json::json!([
                        {
                            "to": CONTRACT,
                            "data": SELECTOR
                        },
                        "latest"
                    ])
                )
            ),
            Some(Duration::from_secs(30))
        );
    }
}
