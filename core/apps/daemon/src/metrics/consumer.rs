use primitives::unix_timestamp;
use std::collections::HashMap;
use std::sync::Mutex;

use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Registry;

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
struct ConsumerLabels {
    consumer: String,
}

#[derive(Default)]
struct ConsumerState {
    total_processed: u64,
    total_skipped: u64,
    total_failed: u64,
    in_flight: u64,
    last_success: Option<u64>,
    last_failure: Option<u64>,
    avg_duration: u64,
    duration_count: u64,
}

pub struct ConsumerMetrics {
    consumers: Mutex<HashMap<String, ConsumerState>>,
    duration: Family<ConsumerLabels, Histogram>,
    queue_wait: Family<ConsumerLabels, Histogram>,
}

impl ConsumerMetrics {
    pub fn new() -> Self {
        Self {
            consumers: Mutex::new(HashMap::new()),
            duration: Family::new_with_constructor(|| Histogram::new([100.0, 250.0, 500.0, 1_000.0, 2_500.0, 5_000.0, 10_000.0])),
            queue_wait: Family::new_with_constructor(|| Histogram::new([1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 900.0])),
        }
    }

    pub fn record_started(&self, name: &str, queue_wait: Option<u64>) {
        let mut consumers = super::locked(&self.consumers);
        consumers.entry(name.to_string()).or_default().in_flight += 1;
        if let Some(queue_wait) = queue_wait {
            self.queue_wait.get_or_create(&ConsumerLabels { consumer: name.to_string() }).observe(queue_wait as f64);
        }
    }

    pub fn record_success(&self, name: &str, duration: u64) {
        let mut consumers = super::locked(&self.consumers);
        let state = consumers.entry(name.to_string()).or_default();
        let timestamp = unix_timestamp();

        state.total_processed += 1;
        state.last_success = Some(timestamp);
        Self::record_finished(state, duration);
        self.observe_duration(name, duration);
    }

    pub fn record_skipped(&self, name: &str, duration: u64) {
        let mut consumers = super::locked(&self.consumers);
        let state = consumers.entry(name.to_string()).or_default();
        state.total_skipped += 1;
        Self::record_finished(state, duration);
        self.observe_duration(name, duration);
    }

    pub fn record_error(&self, name: &str, duration: u64) {
        let mut consumers = super::locked(&self.consumers);
        let state = consumers.entry(name.to_string()).or_default();
        state.total_failed += 1;
        state.last_failure = Some(unix_timestamp());
        Self::record_finished(state, duration);
        self.observe_duration(name, duration);
    }

    fn record_finished(state: &mut ConsumerState, duration: u64) {
        state.in_flight = state.in_flight.saturating_sub(1);
        let previous_count = state.duration_count;
        state.duration_count += 1;
        state.avg_duration = (state.avg_duration * previous_count + duration) / state.duration_count;
    }

    fn observe_duration(&self, name: &str, duration: u64) {
        self.duration.get_or_create(&ConsumerLabels { consumer: name.to_string() }).observe(duration as f64);
    }
}

impl MetricsProvider for ConsumerMetrics {
    fn register(&self, registry: &mut Registry) {
        let processed = Family::<ConsumerLabels, Gauge>::default();
        let skipped = Family::<ConsumerLabels, Gauge>::default();
        let failed = Family::<ConsumerLabels, Gauge>::default();
        let in_flight = Family::<ConsumerLabels, Gauge>::default();
        let last_success_at = Family::<ConsumerLabels, Gauge>::default();
        let last_failure_at = Family::<ConsumerLabels, Gauge>::default();
        let avg_duration = Family::<ConsumerLabels, Gauge>::default();

        let consumers = super::locked(&self.consumers);
        for (name, state) in consumers.iter() {
            let labels = ConsumerLabels { consumer: name.clone() };

            processed.get_or_create(&labels).set(state.total_processed as i64);
            skipped.get_or_create(&labels).set(state.total_skipped as i64);
            failed.get_or_create(&labels).set(state.total_failed as i64);
            in_flight.get_or_create(&labels).set(state.in_flight as i64);
            if let Some(ts) = state.last_success {
                last_success_at.get_or_create(&labels).set(ts as i64);
            }
            if let Some(ts) = state.last_failure {
                last_failure_at.get_or_create(&labels).set(ts as i64);
            }
            avg_duration.get_or_create(&labels).set(state.avg_duration as i64);
        }

        registry.register("consumer_processed", "Messages processed", processed);
        registry.register("consumer_skipped", "Messages skipped", skipped);
        registry.register("consumer_failed", "Messages failed", failed);
        registry.register("consumer_in_flight", "Messages currently being processed", in_flight);
        registry.register("consumer_last_success_at", "Last successful processing (unix timestamp)", last_success_at);
        registry.register("consumer_last_failure_at", "Last failed processing (unix timestamp)", last_failure_at);
        registry.register("consumer_avg_duration_milliseconds", "Average processing duration in milliseconds", avg_duration);
        registry.register("consumer_duration_milliseconds", "Consumer processing duration in milliseconds", self.duration.clone());
        registry.register("consumer_queue_wait_seconds", "Time between message publication and consumer processing", self.queue_wait.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_each_consumer_outcome_and_in_flight() {
        let metrics = ConsumerMetrics::new();

        metrics.record_started("store_transactions.ethereum", Some(12));
        metrics.record_success("store_transactions.ethereum", 120);
        metrics.record_started("store_transactions.ethereum", None);
        metrics.record_skipped("store_transactions.ethereum", 20);
        metrics.record_started("store_transactions.ethereum", Some(15));
        metrics.record_error("store_transactions.ethereum", 220);

        let consumers = super::super::locked(&metrics.consumers);
        let state = consumers.get("store_transactions.ethereum").unwrap();
        assert_eq!(state.total_processed, 1);
        assert_eq!(state.total_skipped, 1);
        assert_eq!(state.total_failed, 1);
        assert_eq!(state.in_flight, 0);
        assert_eq!(state.avg_duration, 120);
        assert!(state.last_success.is_some());
        assert!(state.last_failure.is_some());
    }
}
