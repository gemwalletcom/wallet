use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use metrics::{MetricsRegistry, prometheus_client};
use primitives::ScanProvider;
use prometheus_client::encoding::{EncodeLabelSet, LabelSetEncoder};
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{Histogram, exponential_buckets};
use rocket::response::content::RawText;
use rocket::{State, get};
use security_provider::TransactionScanProviders;

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
}

impl Metrics {
    pub fn new(providers: &TransactionScanProviders) -> Self {
        let scan_latency = Family::<ScanLabels, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(10.0, 2.0, 12)));
        for (provider, kind) in providers
            .addresses
            .iter()
            .map(|provider| (provider.provider(), "address"))
            .chain(providers.poisoning.iter().map(|provider| (provider.provider(), "address_poisoning")))
            .chain(providers.websites.iter().map(|provider| (provider.provider(), "website")))
        {
            for outcome in ["clean", "malicious", "error"] {
                drop(scan_latency.get_or_create(&ScanLabels {
                    provider: provider.as_ref().to_string(),
                    kind,
                    outcome,
                }));
            }
        }
        let mut registry = MetricsRegistry::with_prefix("api");
        registry
            .registry_mut()
            .register("security_scan_latency_milliseconds", "Security provider request latency", scan_latency.clone());
        Self { registry, scan_latency }
    }

    pub fn record_scan(&self, provider: ScanProvider, kind: &'static str, malicious: Option<bool>, latency: Duration) {
        let outcome = match malicious {
            Some(false) => "clean",
            Some(true) => "malicious",
            None => "error",
        };
        self.scan_latency
            .get_or_create(&ScanLabels {
                provider: provider.as_ref().to_string(),
                kind,
                outcome,
            })
            .observe(latency.as_secs_f64() * 1000.0);
    }
}

#[get("/metrics")]
pub fn get_metrics(metrics: &State<Arc<Metrics>>) -> RawText<String> {
    RawText(metrics.registry.encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::routes;

    #[test]
    fn test_metrics_provider_outcomes() {
        let metrics = Arc::new(Metrics::new(&TransactionScanProviders {
            addresses: vec![],
            poisoning: vec![],
            websites: vec![],
        }));
        metrics.record_scan(ScanProvider::HashDit, "address", Some(false), Duration::from_millis(125));
        metrics.record_scan(ScanProvider::HashDit, "address", Some(true), Duration::from_secs(2));
        metrics.record_scan(ScanProvider::HashDit, "website", None, Duration::from_millis(62500));
        metrics.record_scan(ScanProvider::GoPlus, "address", Some(false), Duration::from_millis(5));
        let client = Client::tracked(rocket::build().manage(metrics).mount("/", routes![get_metrics])).unwrap();
        let response = client.get("/metrics").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
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
