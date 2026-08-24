use std::sync::{Arc, atomic::AtomicU64};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use metrics::MetricsRegistry;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};

#[derive(Clone, Debug)]
pub(crate) struct Metrics {
    registry: Arc<MetricsRegistry>,
    requests: Family<RequestLabels, Counter>,
    responses: Family<ResponseLabels, Counter>,
    failovers: Family<FailoverLabels, Counter>,
    inflight: Family<TrafficLabels, Gauge>,
    upstream_latency: Family<UpstreamLabels, Histogram>,
    throttle_wait: Family<EndpointLabels, Histogram>,
    cooldowns: Family<CooldownLabels, TimestampGauge>,
    proxy_available: Family<ProxyLabels, Gauge>,
}

pub(crate) struct Inflight {
    metrics: Metrics,
    labels: TrafficLabels,
}

type TimestampGauge = Gauge<u64, AtomicU64>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct RequestLabels {
    caller: String,
    group: String,
    service: String,
    endpoint: String,
    path: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ResponseLabels {
    caller: String,
    group: String,
    service: String,
    path: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct FailoverLabels {
    caller: String,
    group: String,
    service: String,
    endpoint: String,
    path: String,
    reason: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct TrafficLabels {
    caller: String,
    group: String,
    service: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct UpstreamLabels {
    caller: String,
    group: String,
    service: String,
    endpoint: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct EndpointLabels {
    caller: String,
    group: String,
    service: String,
    endpoint: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct CooldownLabels {
    group: String,
    service: String,
    endpoint: String,
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ProxyLabels {
    proxy: String,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        let requests = Family::<RequestLabels, Counter>::default();
        let responses = Family::<ResponseLabels, Counter>::default();
        let failovers = Family::<FailoverLabels, Counter>::default();
        let inflight = Family::<TrafficLabels, Gauge>::default();
        let upstream_latency = Family::<UpstreamLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(10.0, 2.0, 10)));
        let throttle_wait = Family::<EndpointLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(10.0, 2.0, 10)));
        let cooldowns = Family::<CooldownLabels, TimestampGauge>::default();
        let proxy_available = Family::<ProxyLabels, Gauge>::default();
        let mut registry = MetricsRegistry::with_prefix("egress");
        registry.registry_mut().register("requests", "Upstream requests", requests.clone());
        registry.registry_mut().register("responses", "Client responses", responses.clone());
        registry.registry_mut().register("failovers", "Endpoint failovers", failovers.clone());
        registry.registry_mut().register("inflight", "Requests currently being handled", inflight.clone());
        registry
            .registry_mut()
            .register("upstream_latency_milliseconds", "Upstream response latency", upstream_latency.clone());
        registry
            .registry_mut()
            .register("throttle_wait_milliseconds", "Time spent waiting for an endpoint rate limit", throttle_wait.clone());
        registry
            .registry_mut()
            .register("cooldown_until_seconds", "Endpoint path cooldown expiry time", cooldowns.clone());
        registry
            .registry_mut()
            .register("proxy_available", "Whether an outbound proxy passed its last check", proxy_available.clone());
        Self {
            registry: Arc::new(registry),
            requests,
            responses,
            failovers,
            inflight,
            upstream_latency,
            throttle_wait,
            cooldowns,
            proxy_available,
        }
    }

    pub(crate) fn record_request(&self, caller: &str, group: &str, service: &str, endpoint: &str, path: &str, status: u16) {
        self.requests
            .get_or_create(&RequestLabels {
                caller: caller.to_string(),
                group: group.to_string(),
                service: service.to_string(),
                endpoint: endpoint.to_string(),
                path: path.to_string(),
                status,
            })
            .inc();
    }

    pub(crate) fn record_response(&self, caller: &str, group: &str, service: &str, path: &str, status: u16) {
        self.responses
            .get_or_create(&ResponseLabels {
                caller: caller.to_string(),
                group: group.to_string(),
                service: service.to_string(),
                path: path.to_string(),
                status,
            })
            .inc();
    }

    pub(crate) fn record_failover(&self, caller: &str, group: &str, service: &str, endpoint: &str, path: &str, reason: &str) {
        self.failovers
            .get_or_create(&FailoverLabels {
                caller: caller.to_string(),
                group: group.to_string(),
                service: service.to_string(),
                endpoint: endpoint.to_string(),
                path: path.to_string(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub(crate) fn track_inflight(&self, caller: &str, group: &str, service: &str) -> Inflight {
        let labels = TrafficLabels {
            caller: caller.to_string(),
            group: group.to_string(),
            service: service.to_string(),
        };
        self.inflight.get_or_create(&labels).inc();
        Inflight { metrics: self.clone(), labels }
    }

    pub(crate) fn record_upstream_latency(&self, caller: &str, group: &str, service: &str, endpoint: &str, status: u16, latency: Duration) {
        self.upstream_latency
            .get_or_create(&UpstreamLabels {
                caller: caller.to_string(),
                group: group.to_string(),
                service: service.to_string(),
                endpoint: endpoint.to_string(),
                status,
            })
            .observe(latency.as_secs_f64() * 1000.0);
    }

    pub(crate) fn record_throttle_wait(&self, caller: &str, group: &str, service: &str, endpoint: &str, wait: Duration) {
        self.throttle_wait
            .get_or_create(&EndpointLabels {
                caller: caller.to_string(),
                group: group.to_string(),
                service: service.to_string(),
                endpoint: endpoint.to_string(),
            })
            .observe(wait.as_secs_f64() * 1000.0);
    }

    pub(crate) fn set_cooldown(&self, group: &str, service: &str, endpoint: &str, path: &str, duration: Duration) {
        let until = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_add(duration.as_secs());
        self.cooldowns
            .get_or_create(&CooldownLabels {
                group: group.to_string(),
                service: service.to_string(),
                endpoint: endpoint.to_string(),
                path: path.to_string(),
            })
            .set(until);
    }

    pub(crate) fn set_proxy_available(&self, proxy: &str, available: bool) {
        self.proxy_available.get_or_create(&ProxyLabels { proxy: proxy.to_string() }).set(i64::from(available));
    }

    pub(crate) fn encode(&self) -> String {
        self.registry.encode()
    }
}

impl Drop for Inflight {
    fn drop(&mut self) {
        self.metrics.inflight.get_or_create(&self.labels).dec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric_line<'a>(encoded: &'a str, prefix: &str) -> &'a str {
        encoded.lines().find(|line| line.starts_with(prefix)).unwrap()
    }

    #[test]
    fn test_records_path_for_requests_and_failovers() {
        let metrics = Metrics::new();
        metrics.record_request("consumer", "indexer", "toncenter", "key_1", "/api/v3/wallet/:value", 200);
        metrics.record_response("consumer", "indexer", "toncenter", "/api/v3/wallet/:value", 200);
        metrics.record_failover("consumer", "indexer", "toncenter", "key_1", "/api/v3/wallet/:value", "429");

        let encoded = metrics.encode();
        assert_eq!(
            encoded,
            concat!(
                "# HELP egress_requests Upstream requests.\n",
                "# TYPE egress_requests counter\n",
                "egress_requests_total{caller=\"consumer\",group=\"indexer\",service=\"toncenter\",endpoint=\"key_1\",path=\"/api/v3/wallet/:value\",status=\"200\"} 1\n",
                "# HELP egress_responses Client responses.\n",
                "# TYPE egress_responses counter\n",
                "egress_responses_total{caller=\"consumer\",group=\"indexer\",service=\"toncenter\",path=\"/api/v3/wallet/:value\",status=\"200\"} 1\n",
                "# HELP egress_failovers Endpoint failovers.\n",
                "# TYPE egress_failovers counter\n",
                "egress_failovers_total{caller=\"consumer\",group=\"indexer\",service=\"toncenter\",endpoint=\"key_1\",path=\"/api/v3/wallet/:value\",reason=\"429\"} 1\n",
                "# EOF\n",
            )
        );
    }

    #[test]
    fn test_records_endpoint_state() {
        let metrics = Metrics::new();
        let inflight = metrics.track_inflight("worker", "prices", "jupiter");
        metrics.record_upstream_latency("worker", "prices", "jupiter", "key_1", 200, Duration::from_millis(125));
        metrics.record_throttle_wait("worker", "prices", "jupiter", "key_1", Duration::from_millis(175));
        metrics.set_cooldown("prices", "jupiter", "key_1", "/tokens/v2/tag", Duration::from_secs(60));

        let encoded = metrics.encode();
        assert_eq!(
            metric_line(&encoded, "egress_inflight{"),
            "egress_inflight{caller=\"worker\",group=\"prices\",service=\"jupiter\"} 1"
        );
        assert_eq!(
            metric_line(&encoded, "egress_upstream_latency_milliseconds_sum{"),
            "egress_upstream_latency_milliseconds_sum{caller=\"worker\",group=\"prices\",service=\"jupiter\",endpoint=\"key_1\",status=\"200\"} 125.0"
        );
        assert_eq!(
            metric_line(&encoded, "egress_throttle_wait_milliseconds_sum{"),
            "egress_throttle_wait_milliseconds_sum{caller=\"worker\",group=\"prices\",service=\"jupiter\",endpoint=\"key_1\"} 175.0"
        );
        let cooldown = metric_line(&encoded, "egress_cooldown_until_seconds{");
        let (labels, expiry) = cooldown.split_once("} ").unwrap();
        assert_eq!(
            labels,
            "egress_cooldown_until_seconds{group=\"prices\",service=\"jupiter\",endpoint=\"key_1\",path=\"/tokens/v2/tag\""
        );
        assert_ne!(expiry.parse::<u64>().unwrap(), 0);

        drop(inflight);
        let encoded = metrics.encode();
        assert_eq!(
            metric_line(&encoded, "egress_inflight{"),
            "egress_inflight{caller=\"worker\",group=\"prices\",service=\"jupiter\"} 0"
        );
    }
}
