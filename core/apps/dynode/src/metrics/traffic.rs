use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gem_tracing::path;
use primitives::{Chain, ChainType};
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::gauge::Gauge;

use super::Metrics;

pub(crate) struct Inflight {
    gauge: Gauge,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct RequestLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    endpoint: String,
    path: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct ClientResponseLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    path: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct FailoverLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    endpoint: String,
    path: String,
    reason: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct TrafficLabels {
    source: String,
    group: String,
    service: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct UpstreamLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    endpoint: String,
    status: u16,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct EndpointLabels {
    #[prometheus(flatten)]
    traffic: TrafficLabels,
    endpoint: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct CooldownLabels {
    group: String,
    service: String,
    endpoint: String,
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub(super) struct ProxyLabels {
    proxy: String,
}

impl TrafficLabels {
    pub(super) fn new(source: &str, group: &str, service: &str) -> Self {
        Self {
            source: source.to_string(),
            group: group.to_string(),
            service: service.to_string(),
        }
    }

    pub(super) fn node(source: &str, chain: &str) -> Self {
        let group = Chain::from_str(chain).map(|chain| chain_group(chain).to_string()).unwrap_or_else(|_| chain.to_string());
        Self {
            source: source.to_string(),
            group,
            service: chain.to_string(),
        }
    }
}

fn chain_group(chain: Chain) -> &'static str {
    match chain.chain_type() {
        ChainType::Ethereum => "evm",
        ChainType::Bitcoin => "bitcoin",
        ChainType::Solana => "solana",
        ChainType::Cosmos => "cosmos",
        ChainType::Ton => "ton",
        ChainType::Tron => "tron",
        ChainType::Aptos => "aptos",
        ChainType::Sui => "sui",
        ChainType::Xrp => "xrp",
        ChainType::Near => "near",
        ChainType::Stellar => "stellar",
        ChainType::Algorand => "algorand",
        ChainType::Polkadot => "polkadot",
        ChainType::Cardano => "cardano",
        ChainType::HyperCore => "hypercore",
    }
}

impl Metrics {
    pub(crate) fn track_node_inflight(&self, chain: Chain) -> Inflight {
        self.track_inflight(&self.source, chain_group(chain), chain.as_ref())
    }

    pub(crate) fn record_node_response(&self, chain: Chain, path: &str, status: u16) {
        self.record_response(&self.source, chain_group(chain), chain.as_ref(), path, status);
    }

    pub(crate) fn record_node_upstream(&self, chain: Chain, endpoint: &str, path: &str, status: u16, latency: Duration) {
        self.record_request(&self.source, chain_group(chain), chain.as_ref(), endpoint, path, status);
        self.record_upstream_latency(&self.source, chain_group(chain), chain.as_ref(), endpoint, status, latency);
    }

    pub(crate) fn record_node_failover(&self, chain: Chain, endpoint: &str, path: &str, reason: &str) {
        self.record_failover(&self.source, chain_group(chain), chain.as_ref(), endpoint, path, reason);
    }

    pub(crate) fn record_request(&self, source: &str, group: &str, service: &str, endpoint: &str, path: &str, status: u16) {
        self.requests
            .get_or_create(&RequestLabels {
                traffic: TrafficLabels::new(source, group, service),
                endpoint: endpoint.to_string(),
                path: path::redact(path),
                status,
            })
            .inc();
    }

    pub(crate) fn record_response(&self, source: &str, group: &str, service: &str, path: &str, status: u16) {
        self.responses
            .get_or_create(&ClientResponseLabels {
                traffic: TrafficLabels::new(source, group, service),
                path: path::redact(path),
                status,
            })
            .inc();
    }

    pub(crate) fn record_failover(&self, source: &str, group: &str, service: &str, endpoint: &str, path: &str, reason: &str) {
        self.failovers
            .get_or_create(&FailoverLabels {
                traffic: TrafficLabels::new(source, group, service),
                endpoint: endpoint.to_string(),
                path: path::redact(path),
                reason: reason.to_string(),
            })
            .inc();
    }

    pub(crate) fn track_inflight(&self, source: &str, group: &str, service: &str) -> Inflight {
        let labels = TrafficLabels::new(source, group, service);
        let gauge = self.inflight.get_or_create(&labels).clone();
        gauge.inc();
        Inflight { gauge }
    }

    pub(crate) fn record_upstream_latency(&self, source: &str, group: &str, service: &str, endpoint: &str, status: u16, latency: Duration) {
        self.upstream_latency
            .get_or_create(&UpstreamLabels {
                traffic: TrafficLabels::new(source, group, service),
                endpoint: endpoint.to_string(),
                status,
            })
            .observe(latency.as_secs_f64() * 1000.0);
    }

    pub(crate) fn record_throttle_wait(&self, source: &str, group: &str, service: &str, endpoint: &str, wait: Duration) {
        self.throttle_wait
            .get_or_create(&EndpointLabels {
                traffic: TrafficLabels::new(source, group, service),
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
                path: path::redact(path),
            })
            .set(until);
    }

    pub(crate) fn set_proxy_available(&self, proxy: &str, available: bool) {
        self.proxy_available.get_or_create(&ProxyLabels { proxy: proxy.to_string() }).set(i64::from(available));
    }
}

impl Drop for Inflight {
    fn drop(&mut self) {
        self.gauge.dec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;
    use crate::testkit::config::metrics_config;

    fn metric_lines(encoded: &str, prefix: &str) -> Vec<String> {
        let mut lines = encoded.lines().filter(|line| line.starts_with(prefix)).map(str::to_owned).collect::<Vec<_>>();
        lines.sort();
        lines
    }

    #[test]
    fn test_node_attempts_and_provider_traffic_share_families() {
        let metrics = Metrics::new(MetricsConfig {
            source: "api".into(),
            ..metrics_config()
        });
        metrics.add_proxy_request("ethereum", &["eth_chainId".into(), "eth_blockNumber".into()]);
        metrics.record_node_upstream(Chain::Ethereum, "rpc.example.com", "/?apikey=secret", 200, Duration::from_millis(50));
        metrics.add_proxy_upstream_response("ethereum", "eth_chainId", "rpc.example.com", 200, 50);
        metrics.add_proxy_upstream_response("ethereum", "eth_blockNumber", "rpc.example.com", 200, 50);
        metrics.record_node_response(Chain::Ethereum, "/?apikey=secret", 200);
        metrics.record_node_response(Chain::Ethereum, "/?apikey=secret", 200);
        metrics.record_request("consumer", "indexer", "toncenter", "key_1", "/api/v3/wallet/12345?apikey=secret", 200);
        metrics.record_response("consumer", "indexer", "toncenter", "/api/v3/wallet/12345?apikey=secret", 200);

        let encoded = metrics.get_metrics();
        assert_eq!(
            metric_lines(&encoded, "dynode_requests_total{"),
            vec![
                "dynode_requests_total{source=\"api\",group=\"evm\",service=\"ethereum\",endpoint=\"rpc.example.com\",path=\"/\",status=\"200\"} 1",
                "dynode_requests_total{source=\"consumer\",group=\"indexer\",service=\"toncenter\",endpoint=\"key_1\",path=\"/api/v3/wallet/:number\",status=\"200\"} 1",
            ]
        );
        assert_eq!(
            metric_lines(&encoded, "dynode_responses_total{"),
            vec![
                "dynode_responses_total{source=\"api\",group=\"evm\",service=\"ethereum\",path=\"/\",status=\"200\"} 2",
                "dynode_responses_total{source=\"consumer\",group=\"indexer\",service=\"toncenter\",path=\"/api/v3/wallet/:number\",status=\"200\"} 1",
            ]
        );
        assert_eq!(
            metric_lines(&encoded, "dynode_upstream_latency_milliseconds_count{"),
            vec!["dynode_upstream_latency_milliseconds_count{source=\"api\",group=\"evm\",service=\"ethereum\",endpoint=\"rpc.example.com\",status=\"200\"} 1",]
        );
        assert_eq!(encoded.lines().filter(|line| line.starts_with("# TYPE dynode_requests ")).count(), 1);
        assert_eq!(encoded.lines().filter(|line| line.starts_with("# EOF")).count(), 1);
        assert_eq!(encoded.find("secret"), None);
    }

    #[test]
    fn test_node_groups_follow_chain_families() {
        let metrics = Metrics::new(metrics_config());
        for chain in [
            Chain::Ethereum,
            Chain::Base,
            Chain::Arbitrum,
            Chain::Bitcoin,
            Chain::Litecoin,
            Chain::Cosmos,
            Chain::Osmosis,
        ] {
            metrics.record_node_response(chain, "/", 200);
            metrics.add_cache_hit(chain.as_ref(), "eth_chainId");
        }
        assert_eq!(
            metric_lines(&metrics.get_metrics(), "dynode_responses_total{"),
            vec![
                "dynode_responses_total{source=\"public\",group=\"bitcoin\",service=\"bitcoin\",path=\"/\",status=\"200\"} 1",
                "dynode_responses_total{source=\"public\",group=\"bitcoin\",service=\"litecoin\",path=\"/\",status=\"200\"} 1",
                "dynode_responses_total{source=\"public\",group=\"cosmos\",service=\"cosmos\",path=\"/\",status=\"200\"} 1",
                "dynode_responses_total{source=\"public\",group=\"cosmos\",service=\"osmosis\",path=\"/\",status=\"200\"} 1",
                "dynode_responses_total{source=\"public\",group=\"evm\",service=\"arbitrum\",path=\"/\",status=\"200\"} 1",
                "dynode_responses_total{source=\"public\",group=\"evm\",service=\"base\",path=\"/\",status=\"200\"} 1",
                "dynode_responses_total{source=\"public\",group=\"evm\",service=\"ethereum\",path=\"/\",status=\"200\"} 1",
            ]
        );
        assert_eq!(metric_lines(&metrics.get_metrics(), "dynode_cache_hits_total{").len(), 7);
    }

    #[test]
    fn test_endpoint_state_and_inflight_drop() {
        let metrics = Metrics::new(metrics_config());
        let inflight = metrics.track_inflight("worker", "prices", "jupiter");
        metrics.record_upstream_latency("worker", "prices", "jupiter", "key_1", 200, Duration::from_millis(125));
        metrics.record_throttle_wait("worker", "prices", "jupiter", "key_1", Duration::from_millis(175));
        metrics.set_cooldown("prices", "jupiter", "key_1", "/tokens/v2/tag?apikey=secret", Duration::from_secs(60));
        metrics.set_proxy_available("proxy_1", false);
        metrics.record_failover("worker", "prices", "jupiter", "key_1", "/tokens/v2/tag?apikey=secret", "429");

        let encoded = metrics.get_metrics();
        assert_eq!(
            metric_lines(&encoded, "dynode_inflight{"),
            vec!["dynode_inflight{source=\"worker\",group=\"prices\",service=\"jupiter\"} 1"]
        );
        assert_eq!(
            metric_lines(&encoded, "dynode_upstream_latency_milliseconds_sum{"),
            vec!["dynode_upstream_latency_milliseconds_sum{source=\"worker\",group=\"prices\",service=\"jupiter\",endpoint=\"key_1\",status=\"200\"} 125.0",]
        );
        assert_eq!(
            metric_lines(&encoded, "dynode_throttle_wait_milliseconds_sum{"),
            vec!["dynode_throttle_wait_milliseconds_sum{source=\"worker\",group=\"prices\",service=\"jupiter\",endpoint=\"key_1\"} 175.0",]
        );
        assert_eq!(metric_lines(&encoded, "dynode_proxy_available{"), vec!["dynode_proxy_available{proxy=\"proxy_1\"} 0"]);
        assert_eq!(
            metric_lines(&encoded, "dynode_failovers_total{"),
            vec!["dynode_failovers_total{source=\"worker\",group=\"prices\",service=\"jupiter\",endpoint=\"key_1\",path=\"/tokens/v2/tag\",reason=\"429\"} 1",]
        );
        let cooldown = metric_lines(&encoded, "dynode_cooldown_until_seconds{");
        let (labels, expiry) = cooldown[0].split_once("} ").unwrap();
        assert_eq!(
            labels,
            "dynode_cooldown_until_seconds{group=\"prices\",service=\"jupiter\",endpoint=\"key_1\",path=\"/tokens/v2/tag\""
        );
        assert_ne!(expiry.parse::<u64>().unwrap(), 0);
        assert_eq!(encoded.find("secret"), None);
        drop(inflight);
        assert_eq!(
            metric_lines(&metrics.get_metrics(), "dynode_inflight{"),
            vec!["dynode_inflight{source=\"worker\",group=\"prices\",service=\"jupiter\"} 0"]
        );
    }
}
