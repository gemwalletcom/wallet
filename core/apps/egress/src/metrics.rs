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
    responses: Family<ResponseLabels, Counter>,
    failovers: Family<FailoverLabels, Counter>,
    proxy_available: Family<ProxyLabels, Gauge>,
}

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
struct ProxyLabels {
    proxy: String,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        let requests = Family::<RequestLabels, Counter>::default();
        let responses = Family::<ResponseLabels, Counter>::default();
        let failovers = Family::<FailoverLabels, Counter>::default();
        let proxy_available = Family::<ProxyLabels, Gauge>::default();
        let mut registry = MetricsRegistry::with_prefix("egress");
        registry.registry_mut().register("requests", "Upstream requests", requests.clone());
        registry.registry_mut().register("responses", "Client responses", responses.clone());
        registry.registry_mut().register("failovers", "Endpoint failovers", failovers.clone());
        registry
            .registry_mut()
            .register("proxy_available", "Whether an outbound proxy passed its last check", proxy_available.clone());
        Self {
            registry: Arc::new(registry),
            requests,
            responses,
            failovers,
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
}
