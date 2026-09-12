mod traffic;

use std::{
    sync::{Arc, atomic::AtomicU64},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use gem_tracing::path;
use metrics::MetricsRegistry;
use primitives::NodeStatusState;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use prometheus_client::registry::Registry;

use crate::config::MetricsConfig;
use traffic::{ClientResponseLabels, CooldownLabels, EndpointLabels, FailoverLabels, ProxyLabels, RequestLabels, TrafficLabels, UpstreamLabels};

#[derive(Debug, Clone)]
pub struct Metrics {
    registry: Arc<MetricsRegistry>,
    source: String,
    requests: Family<RequestLabels, Counter>,
    responses: Family<ClientResponseLabels, Counter>,
    failovers: Family<FailoverLabels, Counter>,
    inflight: Family<TrafficLabels, Gauge>,
    upstream_latency: Family<UpstreamLabels, Histogram>,
    throttle_wait: Family<EndpointLabels, Histogram>,
    cooldowns: Family<CooldownLabels, NodeMonitorGauge>,
    proxy_available: Family<ProxyLabels, Gauge>,
    proxy_requests: Family<ProxyRequestLabels, Counter>,
    proxy_requests_by_method: Family<ProxyRequestByMethodLabels, Counter>,
    proxy_response_latency: Family<ResponseLabels, Histogram>,
    proxy_upstream_response_latency: Family<UpstreamResponseLabels, Histogram>,
    proxy_retries: Family<RetryLabels, Counter>,
    node_host_current: Family<NodeLabels, Gauge>,
    node_monitor: NodeMonitorMetrics,
    cache_hits: Family<CacheLabels, Counter>,
    cache_misses: Family<CacheLabels, Counter>,
    node_switches: Family<NodeSwitchLabels, Counter>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ProxyRequestLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ProxyRequestByMethodLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    method: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct NodeLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    host: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct NodeMonitorCycleLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    trigger: String,
}

type NodeMonitorGauge = Gauge<u64, AtomicU64>;

#[derive(Debug, Clone, Default)]
struct NodeMonitorMetrics {
    source: String,
    check_success: Family<NodeLabels, NodeMonitorGauge>,
    in_sync: Family<NodeLabels, NodeMonitorGauge>,
    latest_block: Family<NodeLabels, NodeMonitorGauge>,
    current_block: Family<NodeLabels, NodeMonitorGauge>,
    latency_milliseconds: Family<NodeLabels, NodeMonitorGauge>,
    last_check_timestamp_seconds: Family<NodeLabels, NodeMonitorGauge>,
    cycles: Family<NodeMonitorCycleLabels, Counter>,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct ResponseLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    method: String,
    path: String,
    status: u16,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct UpstreamResponseLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    host: String,
    method: String,
    status: u16,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct RetryLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    host: String,
    reason: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct CacheLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    path: String,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, EncodeLabelSet)]
struct NodeSwitchLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    chain: String,
    old_host: String,
    new_host: String,
    reason: String,
}

impl Metrics {
    pub fn new(config: MetricsConfig) -> Self {
        let requests = Family::<RequestLabels, Counter>::default();
        let responses = Family::<ClientResponseLabels, Counter>::default();
        let failovers = Family::<FailoverLabels, Counter>::default();
        let inflight = Family::<TrafficLabels, Gauge>::default();
        let upstream_latency = Family::<UpstreamLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(10.0, 2.0, 10)));
        let throttle_wait = Family::<EndpointLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(10.0, 2.0, 10)));
        let cooldowns = Family::<CooldownLabels, NodeMonitorGauge>::default();
        let proxy_available = Family::<ProxyLabels, Gauge>::default();
        let proxy_requests = Family::<ProxyRequestLabels, Counter>::default();
        let proxy_requests_by_method = Family::<ProxyRequestByMethodLabels, Counter>::default();
        let proxy_response_latency = Family::<ResponseLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(50.0, 2.0, 6)));
        let proxy_upstream_response_latency = Family::<UpstreamResponseLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(50.0, 2.0, 6)));
        let proxy_retries = Family::<RetryLabels, Counter>::default();
        let node_host_current = Family::<NodeLabels, Gauge>::default();
        let cache_hits = Family::<CacheLabels, Counter>::default();
        let cache_misses = Family::<CacheLabels, Counter>::default();
        let node_switches = Family::<NodeSwitchLabels, Counter>::default();

        let mut metrics_registry = MetricsRegistry::with_prefix(&config.prefix);
        let registry = metrics_registry.registry_mut();
        registry.register("requests", "Upstream requests", requests.clone());
        registry.register("responses", "Client responses", responses.clone());
        registry.register("failovers", "Endpoint failovers", failovers.clone());
        registry.register("inflight", "Requests currently being handled", inflight.clone());
        registry.register("upstream_latency_milliseconds", "Upstream response latency", upstream_latency.clone());
        registry.register("throttle_wait_milliseconds", "Time spent waiting for an endpoint rate limit", throttle_wait.clone());
        registry.register("cooldown_until_seconds", "Endpoint path cooldown expiry time", cooldowns.clone());
        registry.register("proxy_available", "Whether an outbound proxy passed its last check", proxy_available.clone());
        registry.register("proxy_requests", "Proxy requests by host", proxy_requests.clone());
        registry.register(
            "proxy_requests_by_method",
            "Proxy requests by host and method (HTTP path or RPC method)",
            proxy_requests_by_method.clone(),
        );
        registry.register(
            "proxy_response_latency",
            "Proxy responses by chain, method, path, and status",
            proxy_response_latency.clone(),
        );
        registry.register(
            "proxy_upstream_response_latency",
            "Upstream proxy responses by host, path, method, and status",
            proxy_upstream_response_latency.clone(),
        );
        registry.register("proxy_retries", "Proxy retries by chain, upstream host, and reason", proxy_retries.clone());
        registry.register("node_host_current", "Node current host url", node_host_current.clone());
        let node_monitor = NodeMonitorMetrics::register(registry, config.source.clone());
        registry.register("cache_hits", "Cache hits by host and path", cache_hits.clone());
        registry.register("cache_misses", "Cache misses by host and path", cache_misses.clone());
        registry.register("node_switches", "Node switches by chain", node_switches.clone());

        Self {
            registry: Arc::new(metrics_registry),
            source: config.source,
            requests,
            responses,
            failovers,
            inflight,
            upstream_latency,
            throttle_wait,
            cooldowns,
            proxy_available,
            proxy_requests,
            proxy_requests_by_method,
            proxy_response_latency,
            proxy_upstream_response_latency,
            proxy_retries,
            node_host_current,
            node_monitor,
            cache_hits,
            cache_misses,
            node_switches,
        }
    }

    pub fn add_proxy_request(&self, chain: &str, methods: &[String]) {
        self.proxy_requests
            .get_or_create(&ProxyRequestLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
            })
            .inc();

        for method in methods {
            let method = self.truncate_method(method);
            self.proxy_requests_by_method
                .get_or_create(&ProxyRequestByMethodLabels {
                    traffic: TrafficLabels::node(&self.source, chain),
                    chain: chain.to_string(),
                    method,
                })
                .inc();
        }
    }

    pub fn add_proxy_upstream_response(&self, chain: &str, method: &str, host: &str, status: u16, latency: u128) {
        let method = self.truncate_method(method);
        self.proxy_upstream_response_latency
            .get_or_create(&UpstreamResponseLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                host: host.to_string(),
                method,
                status,
            })
            .observe(latency as f64);
    }

    pub fn add_proxy_response(&self, chain: &str, method: &str, path: &str, status: u16, latency: u128) {
        let path = path::redact(path);
        self.proxy_response_latency
            .get_or_create(&ResponseLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                method: method.to_string(),
                path,
                status,
            })
            .observe(latency as f64);
    }

    pub fn add_proxy_retry(&self, chain: &str, host: &str, reason: &str) {
        self.proxy_retries
            .get_or_create(&RetryLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                host: host.to_string(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub fn set_node_host_current(&self, chain: &str, host: &str) {
        self.node_host_current
            .get_or_create(&NodeLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                host: host.to_string(),
            })
            .set(1);
    }

    pub fn move_node_host_current(&self, chain: &str, old_host: &str, new_host: &str) {
        self.node_host_current
            .get_or_create(&NodeLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                host: old_host.to_string(),
            })
            .set(0);
        self.set_node_host_current(chain, new_host);
    }

    pub fn record_node_monitor_observation(&self, chain: &str, host: &str, state: &NodeStatusState, latency: Duration) {
        self.node_monitor.record_observation(chain, host, state, latency);
    }

    pub fn add_node_monitor_cycle(&self, chain: &str, trigger: &str) {
        self.node_monitor.add_cycle(chain, trigger);
    }

    pub fn add_cache_hit(&self, chain: &str, path: &str) {
        let path = path::redact(path);
        self.cache_hits
            .get_or_create(&CacheLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                path,
            })
            .inc();
    }

    pub fn add_cache_miss(&self, chain: &str, path: &str) {
        let path = path::redact(path);
        self.cache_misses
            .get_or_create(&CacheLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                path,
            })
            .inc();
    }

    pub(crate) fn record_cache_hit(&self, source: &str, group: &str, service: &str, path: &str) {
        self.cache_hits
            .get_or_create(&CacheLabels {
                traffic: TrafficLabels::new(source, group, service),
                chain: String::new(),
                path: path::redact(path),
            })
            .inc();
    }

    pub(crate) fn record_cache_miss(&self, source: &str, group: &str, service: &str, path: &str) {
        self.cache_misses
            .get_or_create(&CacheLabels {
                traffic: TrafficLabels::new(source, group, service),
                chain: String::new(),
                path: path::redact(path),
            })
            .inc();
    }

    pub fn add_node_switch(&self, chain: &str, old_host: &str, new_host: &str, reason: &str) {
        self.node_switches
            .get_or_create(&NodeSwitchLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                old_host: old_host.to_string(),
                new_host: new_host.to_string(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub fn get_metrics(&self) -> String {
        self.registry.encode()
    }

    fn truncate_method(&self, method: &str) -> String {
        if method.contains('/') { path::redact(method) } else { method.to_string() }
    }
}

impl NodeMonitorMetrics {
    fn record_observation(&self, chain: &str, host: &str, state: &NodeStatusState, latency: Duration) {
        let labels = NodeLabels {
            traffic: TrafficLabels::node(&self.source, chain),
            chain: chain.to_string(),
            host: host.to_string(),
        };
        let (check_success, in_sync, latest_block, current_block) = match state {
            NodeStatusState::Healthy(status) => (
                1,
                u64::from(status.in_sync),
                status.latest_block_number.or(status.current_block_number),
                status.current_block_number.or(status.latest_block_number),
            ),
            NodeStatusState::Error { .. } => (0, 0, None, None),
        };

        self.check_success.get_or_create(&labels).set(check_success);
        self.in_sync.get_or_create(&labels).set(in_sync);
        self.latest_block.get_or_create(&labels).set(latest_block.unwrap_or_default());
        self.current_block.get_or_create(&labels).set(current_block.unwrap_or_default());
        self.latency_milliseconds.get_or_create(&labels).set(latency.as_millis().try_into().unwrap_or(u64::MAX));
        if let Ok(timestamp) = SystemTime::now().duration_since(UNIX_EPOCH) {
            self.last_check_timestamp_seconds.get_or_create(&labels).set(timestamp.as_secs());
        }
    }

    fn add_cycle(&self, chain: &str, trigger: &str) {
        self.cycles
            .get_or_create(&NodeMonitorCycleLabels {
                traffic: TrafficLabels::node(&self.source, chain),
                chain: chain.to_string(),
                trigger: trigger.to_string(),
            })
            .inc();
    }

    fn register(registry: &mut Registry, source: String) -> Self {
        let metrics = Self { source, ..Self::default() };

        registry.register(
            "node_monitor_check_success",
            "Whether the last node monitor check completed successfully",
            metrics.check_success.clone(),
        );
        registry.register("node_monitor_in_sync", "Whether the node was in sync at the last monitor check", metrics.in_sync.clone());
        registry.register("node_monitor_latest_block", "Latest block reported by the node monitor", metrics.latest_block.clone());
        registry.register("node_monitor_current_block", "Current block reported by the node monitor", metrics.current_block.clone());
        registry.register(
            "node_monitor_latency_milliseconds",
            "Duration of the last node monitor check in milliseconds",
            metrics.latency_milliseconds.clone(),
        );
        registry.register(
            "node_monitor_last_check_timestamp_seconds",
            "Unix timestamp of the last node monitor check",
            metrics.last_check_timestamp_seconds.clone(),
        );
        registry.register("node_monitor_cycles", "Node monitor cycles by trigger", metrics.cycles.clone());

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;
    use crate::testkit::config::metrics_config;

    fn create_test_metrics() -> Metrics {
        let config = MetricsConfig {
            prefix: "test".to_string(),
            ..metrics_config()
        };
        Metrics::new(config)
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
    fn test_records_response_once_and_retries_separately() {
        let metrics = create_test_metrics();

        metrics.add_proxy_response("tron", "POST", "/wallet/getaccount?visible=true", 200, 123);
        metrics.add_proxy_retry("tron", "api.trongrid.io", "status=429");

        let encoded = metrics.get_metrics();
        assert_eq!(
            encoded.lines().filter(|line| line.starts_with("test_proxy_response_latency_count{")).collect::<Vec<_>>(),
            vec![
                "test_proxy_response_latency_count{source=\"public\",group=\"tron\",service=\"tron\",chain=\"tron\",method=\"POST\",path=\"/wallet/getaccount\",status=\"200\"} 1"
            ]
        );
        assert_eq!(
            encoded.lines().filter(|line| line.starts_with("test_proxy_retries_total{")).collect::<Vec<_>>(),
            vec!["test_proxy_retries_total{source=\"public\",group=\"tron\",service=\"tron\",chain=\"tron\",host=\"api.trongrid.io\",reason=\"status=429\"} 1"]
        );
    }

    #[test]
    fn test_moves_current_node_host() {
        let metrics = create_test_metrics();

        metrics.set_node_host_current("thorchain", "thornode.ninerealms.com");
        metrics.move_node_host_current("thorchain", "thornode.ninerealms.com", "gateway.liquify.com");

        let encoded = metrics.get_metrics();
        let mut hosts = encoded.lines().filter(|line| line.starts_with("test_node_host_current{")).collect::<Vec<_>>();
        hosts.sort();
        assert_eq!(
            hosts,
            vec![
                "test_node_host_current{source=\"public\",group=\"cosmos\",service=\"thorchain\",chain=\"thorchain\",host=\"gateway.liquify.com\"} 1",
                "test_node_host_current{source=\"public\",group=\"cosmos\",service=\"thorchain\",chain=\"thorchain\",host=\"thornode.ninerealms.com\"} 0"
            ]
        );
    }
}
