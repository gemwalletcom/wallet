use primitives::unix_timestamp;
use std::collections::HashMap;
use std::sync::Mutex;

use metrics::{Family, Gauge, LabelEncoder, LabelSet, MetricsRegistry};

use super::MetricsProvider;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct JobLabels {
    service: String,
    job_name: String,
}

impl LabelSet for JobLabels {
    fn encode(&self, encoder: &mut LabelEncoder<'_>) {
        encoder.add("service", &self.service);
        encoder.add("job_name", &self.job_name);
    }
}

#[derive(Default)]
struct JobState {
    interval: u64,
    duration: Option<u64>,
    last_success: Option<u64>,
}

pub struct JobMetrics {
    service: String,
    jobs: Mutex<HashMap<String, JobState>>,
}

impl JobMetrics {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn restore(&self, name: &str, interval: u64, last_success_at: Option<u64>) {
        let mut jobs = super::locked(&self.jobs);
        let state = jobs.entry(name.to_string()).or_default();

        state.interval = interval;
        state.last_success = state.last_success.max(last_success_at);
    }

    pub fn report(&self, name: &str, interval: u64, duration: u64, success: bool) {
        let mut jobs = super::locked(&self.jobs);
        let state = jobs.entry(name.to_string()).or_default();

        state.interval = interval;
        state.duration = Some(duration);

        if success {
            let timestamp = unix_timestamp();
            state.last_success = Some(timestamp);
        }
    }
}

impl MetricsProvider for JobMetrics {
    fn register(&self, registry: &mut MetricsRegistry) {
        let last_success_at = Family::<JobLabels, Gauge>::default();
        let interval = Family::<JobLabels, Gauge>::default();
        let duration = Family::<JobLabels, Gauge>::default();

        let jobs = super::locked(&self.jobs);
        for (name, state) in jobs.iter() {
            let labels = JobLabels {
                service: self.service.clone(),
                job_name: name.clone(),
            };

            interval.get_or_create(&labels).set(state.interval as i64);
            if let Some(milliseconds) = state.duration {
                duration.get_or_create(&labels).set(milliseconds as i64);
            }

            if let Some(ts) = state.last_success {
                last_success_at.get_or_create(&labels).set(ts as i64);
            }
        }

        registry.register("job_last_success_at", "Last successful job run (unix timestamp)", last_success_at);
        registry.register("job_interval_seconds", "Job interval in seconds", interval);
        registry.register("job_duration_milliseconds", "Last job duration in milliseconds", duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_reports_persisted_last_success_until_a_newer_one() {
        let metrics = JobMetrics::new("alerter");
        metrics.restore("failing", 21600, Some(1_700_000_000));
        metrics.report("failing", 21600, 10, false);
        metrics.report("succeeded", 60, 10, true);
        metrics.restore("succeeded", 60, Some(1_700_000_000));
        metrics.restore("never", 60, None);

        let mut registry = MetricsRegistry::new();
        metrics.register(&mut registry);
        let output = registry.encode();

        assert!(output.contains(r#"job_last_success_at{service="alerter",job_name="failing"} 1700000000"#));
        assert!(!output.contains(r#"job_last_success_at{service="alerter",job_name="succeeded"} 1700000000"#));
        assert!(output.contains(r#"job_interval_seconds{service="alerter",job_name="never"} 60"#));
        assert!(!output.contains(r#"job_last_success_at{service="alerter",job_name="never"}"#));
        assert!(!output.contains(r#"job_duration_milliseconds{service="alerter",job_name="never"}"#));
    }
}
