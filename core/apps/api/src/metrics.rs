use std::fmt;
use std::time::Duration;

use http_server::HttpMetrics;
use metrics::{MetricsRegistry, prometheus_client};
use primitives::{ScanOutcome, ScanProvider, ScanType};
use prometheus_client::encoding::{EncodeLabelSet, LabelSetEncoder};
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use security::TransactionScanProviders;
use services::security::ScanMetrics;

const METRICS_PREFIX: &str = "api";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct ScanLabels {
    provider: String,
    kind: &'static str,
    outcome: &'static str,
}

impl EncodeLabelSet for ScanLabels {
    fn encode(&self, encoder: &mut LabelSetEncoder) -> Result<(), fmt::Error> {
        [("provider", self.provider.as_str()), ("kind", self.kind), ("outcome", self.outcome)].encode(encoder)
    }
}

pub struct Metrics {
    registry: MetricsRegistry,
    scan_latency: Family<ScanLabels, Histogram>,
    http: HttpMetrics,
}

impl Metrics {
    pub fn new(providers: &TransactionScanProviders) -> Self {
        let scan_latency = Family::<ScanLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(10.0, 2.0, 12)));
        for (provider, kind) in providers
            .addresses
            .iter()
            .map(|provider| (provider.provider(), ScanType::Address))
            .chain(providers.poisoning.iter().map(|provider| (provider.provider(), ScanType::AddressPoisoning)))
            .chain(providers.websites.iter().map(|provider| (provider.provider(), ScanType::Website)))
        {
            for outcome in ScanOutcome::all() {
                drop(scan_latency.get_or_create(&ScanLabels {
                    provider: provider.as_ref().to_string(),
                    kind: kind.into(),
                    outcome: outcome.into(),
                }));
            }
        }
        let mut registry = MetricsRegistry::with_prefix(METRICS_PREFIX);
        registry.registry_mut().register("security_scan_latency_milliseconds", "Security provider request latency", scan_latency.clone());
        let http = HttpMetrics::new();
        http.register(registry.registry_mut());
        Self { registry, scan_latency, http }
    }

    pub fn http(&self) -> &HttpMetrics {
        &self.http
    }

    pub fn encode(&self) -> String {
        self.registry.encode()
    }
}

impl ScanMetrics for Metrics {
    fn record_scan(&self, provider: ScanProvider, scan_type: ScanType, outcome: ScanOutcome, latency: Duration) {
        self.scan_latency
            .get_or_create(&ScanLabels {
                provider: provider.as_ref().to_string(),
                kind: scan_type.into(),
                outcome: outcome.into(),
            })
            .observe(latency.as_secs_f64() * 1000.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_provider_outcomes() {
        let metrics = Metrics::new(&TransactionScanProviders {
            addresses: vec![],
            poisoning: vec![],
            websites: vec![],
        });
        metrics.record_scan(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Clean, Duration::from_millis(125));
        metrics.record_scan(ScanProvider::HashDit, ScanType::Address, ScanOutcome::Malicious, Duration::from_secs(2));
        metrics.record_scan(ScanProvider::HashDit, ScanType::Website, ScanOutcome::Error, Duration::from_millis(62500));
        metrics.record_scan(ScanProvider::GoPlus, ScanType::Address, ScanOutcome::Clean, Duration::from_millis(5));
        let body = metrics.encode();
        let mut samples: Vec<_> = body
            .lines()
            .filter(|line| line.starts_with("api_security_scan_latency_milliseconds_count{") || line.starts_with("api_security_scan_latency_milliseconds_sum{"))
            .collect();
        samples.sort_unstable();
        assert_eq!(
            samples,
            vec![
                "api_security_scan_latency_milliseconds_count{provider=\"goplus\",kind=\"address\",outcome=\"clean\"} 1",
                "api_security_scan_latency_milliseconds_count{provider=\"hashdit\",kind=\"address\",outcome=\"clean\"} 1",
                "api_security_scan_latency_milliseconds_count{provider=\"hashdit\",kind=\"address\",outcome=\"malicious\"} 1",
                "api_security_scan_latency_milliseconds_count{provider=\"hashdit\",kind=\"website\",outcome=\"error\"} 1",
                "api_security_scan_latency_milliseconds_sum{provider=\"goplus\",kind=\"address\",outcome=\"clean\"} 5.0",
                "api_security_scan_latency_milliseconds_sum{provider=\"hashdit\",kind=\"address\",outcome=\"clean\"} 125.0",
                "api_security_scan_latency_milliseconds_sum{provider=\"hashdit\",kind=\"address\",outcome=\"malicious\"} 2000.0",
                "api_security_scan_latency_milliseconds_sum{provider=\"hashdit\",kind=\"website\",outcome=\"error\"} 62500.0",
            ]
        );
    }
}
