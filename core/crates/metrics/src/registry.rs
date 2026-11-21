use crate::encoder::{Metric, MetricEncoder, TOTAL_SUFFIX};
use crate::family::{EncodeFamily, Family};
use crate::labels::LabelSet;

const EOF_MARKER: &str = "# EOF\n";

#[derive(Debug, Default)]
pub struct MetricsRegistry {
    prefix: Option<String>,
    families: Vec<Registration>,
    sub_registries: Vec<MetricsRegistry>,
}

#[derive(Debug)]
struct Registration {
    name: &'static str,
    help: &'static str,
    family: Box<dyn EncodeFamily>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: Some(prefix.into()),
            ..Self::default()
        }
    }

    pub fn register<S: LabelSet, M: Metric>(&mut self, name: &'static str, help: &'static str, family: Family<S, M>) {
        debug_assert!(
            !M::TYPE.appends_total_suffix() || !name.ends_with(TOTAL_SUFFIX),
            "counter name {name} must not end with {TOTAL_SUFFIX}, the suffix is added during encoding"
        );
        self.families.push(Registration { name, help, family: Box::new(family) });
    }

    pub fn sub_registry_with_prefix(&mut self, prefix: &str) -> &mut Self {
        let prefix = match &self.prefix {
            Some(parent) => format!("{parent}_{prefix}"),
            None => prefix.to_string(),
        };
        self.sub_registries.push_mut(Self::with_prefix(prefix))
    }

    pub fn encode(&self) -> String {
        let mut out = String::new();
        self.encode_families(&mut out);
        out.push_str(EOF_MARKER);
        out
    }

    fn encode_families(&self, out: &mut String) {
        for registration in &self.families {
            registration.family.encode(&mut MetricEncoder::new(out, self.prefix.as_deref(), registration.name, registration.help));
        }
        for registry in &self.sub_registries {
            registry.encode_families(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::LabelPairs;
    use crate::{Counter, Gauge, Histogram, exponential_buckets};

    #[test]
    fn test_encode() {
        let mut registry = MetricsRegistry::new();
        let first = Family::<LabelPairs, Counter>::default();
        let second = Family::<LabelPairs, Counter>::default();
        let third = Family::<LabelPairs, Counter>::default();
        registry.register("counter1", "First counter", first.clone());
        registry.register("counter2", "Second counter", second.clone());
        registry.register("counter3", "Third counter", third);
        assert_eq!(registry.encode(), "# EOF\n");

        first.get_or_create(&LabelPairs::new(&[("label", "value")])).inc();
        assert_eq!(registry.encode(), "# HELP counter1 First counter.\n# TYPE counter1 counter\ncounter1_total{label=\"value\"} 1\n# EOF\n");

        second.get_or_create(&LabelPairs::new(&[("label", "value")])).inc();
        assert_eq!(
            registry.encode(),
            concat!(
                "# HELP counter1 First counter.\n",
                "# TYPE counter1 counter\n",
                "counter1_total{label=\"value\"} 1\n",
                "# HELP counter2 Second counter.\n",
                "# TYPE counter2 counter\n",
                "counter2_total{label=\"value\"} 1\n",
                "# EOF\n",
            )
        );

        let mut registry = MetricsRegistry::with_prefix("api");
        let gauges = Family::<LabelPairs, Gauge>::default();
        let unsigned = Family::<LabelPairs, Gauge<u64>>::default();
        let histograms = Family::<LabelPairs, Histogram>::new_with_constructor(|| Histogram::new([1.0, 2.0]));
        registry.register("inflight", "Requests in flight", gauges.clone());
        registry.register("block", "Latest block", unsigned.clone());
        registry.register("latency_seconds", "Latency", histograms.clone());
        let gauge = gauges.get_or_create(&LabelPairs::new(&[("chain", "ethereum")]));
        gauge.inc();
        gauge.inc();
        gauge.dec();
        gauges.get_or_create(&LabelPairs::new(&[])).set(-42);
        unsigned.get_or_create(&LabelPairs::new(&[("chain", "ethereum")])).set(u64::MAX);
        histograms.get_or_create(&LabelPairs::new(&[])).observe(1.0);
        let mut lines = registry.encode().lines().map(str::to_owned).collect::<Vec<_>>();
        lines[2..4].sort();
        assert_eq!(
            lines,
            vec![
                "# HELP api_inflight Requests in flight.",
                "# TYPE api_inflight gauge",
                "api_inflight -42",
                "api_inflight{chain=\"ethereum\"} 1",
                "# HELP api_block Latest block.",
                "# TYPE api_block gauge",
                "api_block{chain=\"ethereum\"} 18446744073709551615",
                "# HELP api_latency_seconds Latency.",
                "# TYPE api_latency_seconds histogram",
                "api_latency_seconds_sum 1.0",
                "api_latency_seconds_count 1",
                "api_latency_seconds_bucket{le=\"1.0\"} 1",
                "api_latency_seconds_bucket{le=\"2.0\"} 1",
                "api_latency_seconds_bucket{le=\"+Inf\"} 1",
                "# EOF",
            ]
        );

        let mut registry = MetricsRegistry::new();
        let histograms = Family::<LabelPairs, Histogram>::new_with_constructor(|| Histogram::new(exponential_buckets(0.001, 2.0, 3)));
        registry.register("http_request_duration_seconds", "HTTP request duration", histograms.clone());
        let labels = LabelPairs::new(&[("method", "GET"), ("route", "/")]);
        histograms.get_or_create(&labels).observe(0.0015);
        histograms.get_or_create(&labels).observe(0.3);
        assert_eq!(
            registry.encode(),
            concat!(
                "# HELP http_request_duration_seconds HTTP request duration.\n",
                "# TYPE http_request_duration_seconds histogram\n",
                "http_request_duration_seconds_sum{method=\"GET\",route=\"/\"} 0.3015\n",
                "http_request_duration_seconds_count{method=\"GET\",route=\"/\"} 2\n",
                "http_request_duration_seconds_bucket{le=\"0.001\",method=\"GET\",route=\"/\"} 0\n",
                "http_request_duration_seconds_bucket{le=\"0.002\",method=\"GET\",route=\"/\"} 1\n",
                "http_request_duration_seconds_bucket{le=\"0.004\",method=\"GET\",route=\"/\"} 1\n",
                "http_request_duration_seconds_bucket{le=\"+Inf\",method=\"GET\",route=\"/\"} 2\n",
                "# EOF\n",
            )
        );

        let mut registry = MetricsRegistry::new();
        let requests = Family::<LabelPairs, Counter>::default();
        registry.register("requests", "Requests", requests.clone());
        requests.get_or_create(&LabelPairs::new(&[("client_version", "a\"} evil{x=\"1\\line\nnext")])).inc();
        assert_eq!(
            registry.encode(),
            concat!("# HELP requests Requests.\n", "# TYPE requests counter\n", r#"requests_total{client_version="a\"} evil{x=\"1\\line\nnext"} 1"#, "\n# EOF\n")
        );

        let value = "v".repeat(3 * 4096);
        let mut registry = MetricsRegistry::new();
        let requests = Family::<LabelPairs, Counter>::default();
        registry.register("requests", "Requests", requests.clone());
        requests.get_or_create(&LabelPairs::new(&[("label", &value)])).inc();
        assert_eq!(registry.encode(), format!("# HELP requests Requests.\n# TYPE requests counter\nrequests_total{{label=\"{value}\"}} 1\n# EOF\n"));
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "counter name my_counter_total must not end with _total"))]
    fn test_register() {
        let mut registry = MetricsRegistry::new();
        registry.register("my_gauge_total", "Gauge", Family::<LabelPairs, Gauge>::default());
        registry.register("my_counter_total", "Counter", Family::<LabelPairs, Counter>::default());
    }

    #[test]
    fn test_sub_registry_with_prefix() {
        let counter = Family::<LabelPairs, Counter>::default();
        counter.get_or_create(&LabelPairs::new(&[]));
        let mut registry = MetricsRegistry::new();
        registry.register("my_top_level_metric", "some help", counter.clone());
        let prefix_1 = registry.sub_registry_with_prefix("prefix_1");
        prefix_1.register("my_prefix_1_metric", "some help", counter.clone());
        prefix_1.sub_registry_with_prefix("prefix_1_1").register("my_prefix_1_1_metric", "some help", counter.clone());
        prefix_1.register("my_prefix_1_late_metric", "some help", counter.clone());
        registry.sub_registry_with_prefix("prefix_2");
        registry.sub_registry_with_prefix("prefix_3").register("my_prefix_3_metric", "some help", counter.clone());
        registry.register("my_top_level_late_metric", "some help", counter);

        assert_eq!(
            registry.encode(),
            concat!(
                "# HELP my_top_level_metric some help.\n",
                "# TYPE my_top_level_metric counter\n",
                "my_top_level_metric_total 0\n",
                "# HELP my_top_level_late_metric some help.\n",
                "# TYPE my_top_level_late_metric counter\n",
                "my_top_level_late_metric_total 0\n",
                "# HELP prefix_1_my_prefix_1_metric some help.\n",
                "# TYPE prefix_1_my_prefix_1_metric counter\n",
                "prefix_1_my_prefix_1_metric_total 0\n",
                "# HELP prefix_1_my_prefix_1_late_metric some help.\n",
                "# TYPE prefix_1_my_prefix_1_late_metric counter\n",
                "prefix_1_my_prefix_1_late_metric_total 0\n",
                "# HELP prefix_1_prefix_1_1_my_prefix_1_1_metric some help.\n",
                "# TYPE prefix_1_prefix_1_1_my_prefix_1_1_metric counter\n",
                "prefix_1_prefix_1_1_my_prefix_1_1_metric_total 0\n",
                "# HELP prefix_3_my_prefix_3_metric some help.\n",
                "# TYPE prefix_3_my_prefix_3_metric counter\n",
                "prefix_3_my_prefix_3_metric_total 0\n",
                "# EOF\n",
            )
        );

        let mut registry = MetricsRegistry::with_prefix("dynode");
        registry.sub_registry_with_prefix("http").register("requests", "HTTP requests", Family::<LabelPairs, Counter>::default());
        let requests = Family::<LabelPairs, Counter>::default();
        requests.get_or_create(&LabelPairs::new(&[]));
        registry.sub_registry_with_prefix("daemon").register("requests", "HTTP requests", requests);
        assert_eq!(
            registry.encode(),
            "# HELP dynode_daemon_requests HTTP requests.\n# TYPE dynode_daemon_requests counter\ndynode_daemon_requests_total 0\n# EOF\n"
        );
    }
}
