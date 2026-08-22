use std::sync::Arc;

use metrics::MetricsRegistry;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;

#[derive(Clone, Debug)]
pub(crate) struct Metrics {
    registry: Arc<MetricsRegistry>,
    requests: Family<RequestLabels, Counter>,
    failovers: Family<FailoverLabels, Counter>,
    proxy_available: Family<ProxyLabels, Gauge>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct RequestLabels {
    route: String,
    endpoint: String,
    path: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct FailoverLabels {
    route: String,
    endpoint: String,
    path: String,
    reason: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ProxyLabels {
    proxy: String,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        let requests = Family::<RequestLabels, Counter>::default();
        let failovers = Family::<FailoverLabels, Counter>::default();
        let proxy_available = Family::<ProxyLabels, Gauge>::default();
        let mut registry = MetricsRegistry::with_prefix("egress");
        registry.registry_mut().register("requests", "Upstream requests", requests.clone());
        registry.registry_mut().register("failovers", "Endpoint failovers", failovers.clone());
        registry
            .registry_mut()
            .register("proxy_available", "Whether an outbound proxy passed its last check", proxy_available.clone());
        Self {
            registry: Arc::new(registry),
            requests,
            failovers,
            proxy_available,
        }
    }

    pub(crate) fn record_request(&self, route: &str, endpoint: &str, path: &str, status: u16) {
        self.requests
            .get_or_create(&RequestLabels {
                route: route.to_string(),
                endpoint: endpoint.to_string(),
                path: path.to_string(),
                status,
            })
            .inc();
    }

    pub(crate) fn record_failover(&self, route: &str, endpoint: &str, path: &str, reason: &str) {
        self.failovers
            .get_or_create(&FailoverLabels {
                route: route.to_string(),
                endpoint: endpoint.to_string(),
                path: path.to_string(),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub(crate) fn set_proxy_available(&self, proxy: &str, available: bool) {
        self.proxy_available.get_or_create(&ProxyLabels { proxy: proxy.to_string() }).set(i64::from(available));
    }

    pub(crate) fn encode(&self) -> String {
        self.registry.encode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_path_for_requests_and_failovers() {
        let metrics = Metrics::new();
        metrics.record_request("toncenter", "toncenter", "/toncenter/api/v3/wallet/:value", 200);
        metrics.record_failover("toncenter", "toncenter", "/toncenter/api/v3/wallet/:value", "429");

        let encoded = metrics.encode();
        assert!(encoded.contains("egress_requests_total{route=\"toncenter\",endpoint=\"toncenter\",path=\"/toncenter/api/v3/wallet/:value\",status=\"200\"} 1"));
        assert!(encoded.contains("egress_failovers_total{route=\"toncenter\",endpoint=\"toncenter\",path=\"/toncenter/api/v3/wallet/:value\",reason=\"429\"} 1"));
    }
}
