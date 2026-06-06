use crate::config::MetricsConfig;
use metrics::MetricsRegistry;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Metrics {
    registry: Arc<MetricsRegistry>,
    proxy_requests: Family<ProxyRequestLabels, Counter>,
    proxy_requests_by_method: Family<ProxyRequestByMethodLabels, Counter>,
    proxy_response_latency: Family<ResponseLabels, Histogram>,
    proxy_upstream_response_latency: Family<UpstreamResponseLabels, Histogram>,
    proxy_retries: Family<RetryLabels, Counter>,
    node_host_current: Family<HostCurrentStateLabels, Gauge>,
    cache_hits: Family<CacheLabels, Counter>,
    cache_misses: Family<CacheLabels, Counter>,
    node_switches: Family<NodeSwitchLabels, Counter>,
    auth_requests: Family<AuthLabels, Counter>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestLabels {
    chain: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProxyRequestByMethodLabels {
    chain: String,
    method: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HostCurrentStateLabels {
    chain: String,
    host: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct ResponseLabels {
    chain: String,
    status: u16,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct UpstreamResponseLabels {
    chain: String,
    host: String,
    method: String,
    status: u16,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct RetryLabels {
    chain: String,
    host: String,
    reason: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct CacheLabels {
    chain: String,
    path: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
pub struct NodeSwitchLabels {
    chain: String,
    old_host: String,
    new_host: String,
    reason: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct AuthLabels {
    auth_status: String,
}

impl Metrics {
    pub fn new(config: MetricsConfig) -> Self {
        let proxy_requests = Family::<ProxyRequestLabels, Counter>::default();
        let proxy_requests_by_method = Family::<ProxyRequestByMethodLabels, Counter>::default();
        let proxy_response_latency = Family::<ResponseLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(50.0, 2.0, 6)));
        let proxy_upstream_response_latency = Family::<UpstreamResponseLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(50.0, 2.0, 6)));
        let proxy_retries = Family::<RetryLabels, Counter>::default();
        let node_host_current = Family::<HostCurrentStateLabels, Gauge>::default();
        let cache_hits = Family::<CacheLabels, Counter>::default();
        let cache_misses = Family::<CacheLabels, Counter>::default();
        let node_switches = Family::<NodeSwitchLabels, Counter>::default();
        let auth_requests = Family::<AuthLabels, Counter>::default();

        let mut metrics_registry = MetricsRegistry::with_prefix(&config.prefix);
        let registry = metrics_registry.registry_mut();
        registry.register("proxy_requests", "Proxy requests by host", proxy_requests.clone());
        registry.register(
            "proxy_requests_by_method",
            "Proxy requests by host and method (HTTP path or RPC method)",
            proxy_requests_by_method.clone(),
        );
        registry.register("proxy_response_latency", "Proxy responses by chain and status", proxy_response_latency.clone());
        registry.register(
            "proxy_upstream_response_latency",
            "Upstream proxy responses by host, path, method, and status",
            proxy_upstream_response_latency.clone(),
        );
        registry.register("proxy_retries", "Proxy retries by chain, upstream host, and reason", proxy_retries.clone());
        registry.register("node_host_current", "Node current host url", node_host_current.clone());
        registry.register("cache_hits", "Cache hits by host and path", cache_hits.clone());
        registry.register("cache_misses", "Cache misses by host and path", cache_misses.clone());
        registry.register("node_switches", "Node switches by chain", node_switches.clone());
        registry.register("auth_requests", "Auth requests by status", auth_requests.clone());

        Self {
            registry: Arc::new(metrics_registry),
            proxy_requests,
            proxy_requests_by_method,
            proxy_response_latency,
            proxy_upstream_response_latency,
            proxy_retries,
            node_host_current,
            cache_hits,
            cache_misses,
            node_switches,
            auth_requests,
        }
    }

    pub fn add_proxy_request(&self, chain: &str) {
        self.proxy_requests.get_or_create(&ProxyRequestLabels { chain: chain.to_string() }).inc();
    }

    pub fn add_proxy_request_by_method(&self, chain: &str, method: &str) {
        let method = self.truncate_method(method);
        self.proxy_requests_by_method
            .get_or_create(&ProxyRequestByMethodLabels { chain: chain.to_string(), method })
            .inc();
    }

    pub fn add_proxy_request_batch(&self, chain: &str, methods: &[String]) {
        self.proxy_requests.get_or_create(&ProxyRequestLabels { chain: chain.to_string() }).inc();

        for method in methods {
            let method = self.truncate_method(method);
            self.proxy_requests_by_method
                .get_or_create(&ProxyRequestByMethodLabels { chain: chain.to_string(), method })
                .inc();
        }
    }

    pub fn add_proxy_upstream_response(&self, chain: &str, method: &str, host: &str, status: u16, latency: u128) {
        let method = self.truncate_method(method);
        self.proxy_upstream_response_latency
            .get_or_create(&UpstreamResponseLabels {
                chain: chain.to_string(),
                host: host.to_string(),
                method,
                status,
            })
            .observe(latency as f64);
    }

    pub fn add_proxy_response(&self, chain: &str, status: u16, latency: u128) {
        self.proxy_response_latency
            .get_or_create(&ResponseLabels { chain: chain.to_string(), status })
            .observe(latency as f64);
    }

    pub fn add_proxy_retry(&self, chain: &str, host: &str, reason: &str) {
        self.proxy_retries
            .get_or_create(&RetryLabels {
                chain: chain.to_string(),
                host: host.to_string(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub fn set_node_host_current(&self, chain: &str, host: &str) {
        self.node_host_current
            .get_or_create(&HostCurrentStateLabels {
                chain: chain.to_string(),
                host: host.to_string(),
            })
            .set(1);
    }

    pub fn move_node_host_current(&self, chain: &str, old_host: &str, new_host: &str) {
        self.node_host_current
            .get_or_create(&HostCurrentStateLabels {
                chain: chain.to_string(),
                host: old_host.to_string(),
            })
            .set(0);
        self.set_node_host_current(chain, new_host);
    }

    pub fn add_cache_hit(&self, chain: &str, path: &str) {
        let path = self.truncate_path(path);
        self.cache_hits.get_or_create(&CacheLabels { chain: chain.to_string(), path }).inc();
    }

    pub fn add_cache_miss(&self, chain: &str, path: &str) {
        let path = self.truncate_path(path);
        self.cache_misses.get_or_create(&CacheLabels { chain: chain.to_string(), path }).inc();
    }

    pub fn add_node_switch(&self, chain: &str, old_host: &str, new_host: &str, reason: &str) {
        self.node_switches
            .get_or_create(&NodeSwitchLabels {
                chain: chain.to_string(),
                old_host: old_host.to_string(),
                new_host: new_host.to_string(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub fn add_auth_request(&self, auth_status: &str) {
        self.auth_requests
            .get_or_create(&AuthLabels {
                auth_status: auth_status.to_string(),
            })
            .inc();
    }

    pub fn get_metrics(&self) -> String {
        self.registry.encode()
    }

    fn truncate_method(&self, method: &str) -> String {
        if method.contains('/') { self.truncate_path(method) } else { method.to_string() }
    }

    fn truncate_path(&self, path: &str) -> String {
        let path_part = path.split_once('?').map(|(p, _)| p).unwrap_or(path);

        path_part
            .split('/')
            .map(|segment| {
                if segment.is_empty() {
                    segment.to_string()
                } else if segment.chars().all(|c| c.is_ascii_digit()) {
                    ":number".to_string()
                } else if segment.len() > 20 {
                    ":value".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;

    fn create_test_metrics() -> Metrics {
        let config = MetricsConfig { prefix: "test".to_string() };
        Metrics::new(config)
    }

    #[test]
    fn test_truncate_path() {
        let metrics = create_test_metrics();

        let test_cases = vec![
            ("/api/v1/verylongsegmentthatisgreaterthan20characters/data", "/api/v1/:value/data"),
            ("/block/12345/transactions", "/block/:number/transactions"),
            ("/block/12345/tx/67890", "/block/:number/tx/:number"),
            ("/api/v1/data", "/api/v1/data"),
            ("/api//data", "/api//data"),
            // Query params are stripped
            ("/api/v2/block/5897744?page=1", "/api/v2/block/:number"),
            ("/thorchain/quote/swap?from=X&to=Y", "/thorchain/quote/swap"),
        ];

        for (input, expected) in test_cases {
            let result = metrics.truncate_path(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_truncate_method() {
        let m = create_test_metrics();

        assert_eq!(m.truncate_method("eth_getBlockByNumber"), "eth_getBlockByNumber");
        assert_eq!(m.truncate_method("eth_getBalance"), "eth_getBalance");
        assert_eq!(m.truncate_method("/api/v1/blocks/by_height/12345"), "/api/v1/blocks/by_height/:number");
        assert_eq!(m.truncate_method("/v1/verylongsegmentthatisgreaterthan20characters"), "/v1/:value");
    }

    #[test]
    fn records_response_once_and_retries_separately() {
        let metrics = create_test_metrics();

        metrics.add_proxy_response("tron", 200, 123);
        metrics.add_proxy_retry("tron", "api.trongrid.io", "status=429");

        let encoded = metrics.get_metrics();
        assert!(encoded.contains("test_proxy_response_latency_count"));
        assert!(encoded.contains("chain=\"tron\""));
        assert!(encoded.contains("status=\"200\""));
        assert!(encoded.contains("test_proxy_retries_total"));
        assert!(encoded.contains("host=\"api.trongrid.io\""));
        assert!(encoded.contains("reason=\"status=429\""));
    }

    #[test]
    fn moves_current_node_host() {
        let metrics = create_test_metrics();

        metrics.set_node_host_current("thorchain", "thornode.ninerealms.com");
        metrics.move_node_host_current("thorchain", "thornode.ninerealms.com", "gateway.liquify.com");

        let encoded = metrics.get_metrics();
        assert!(encoded.contains("test_node_host_current{chain=\"thorchain\",host=\"thornode.ninerealms.com\"} 0"));
        assert!(encoded.contains("test_node_host_current{chain=\"thorchain\",host=\"gateway.liquify.com\"} 1"));
    }
}
