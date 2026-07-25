use std::{collections::VecDeque, time::Duration};

use tokio::time::Instant;

use crate::config::FailureTriggerConfig;

const MAX_BUCKETS: u32 = 60;

struct FailureBucket {
    started_at: Instant,
    requests: usize,
    failures: usize,
}

#[derive(Default)]
pub(super) struct FailureTracker {
    buckets: VecDeque<FailureBucket>,
    requests: usize,
    failures: usize,
    consecutive_failures: usize,
    last_triggered_at: Option<Instant>,
}

impl FailureTracker {
    pub(super) fn record(&mut self, failed: bool, now: Instant, config: &FailureTriggerConfig) -> bool {
        self.prune(now, config.window);
        self.record_sample(failed, now, config.window);
        self.requests += 1;
        if failed {
            self.failures += 1;
            self.consecutive_failures += 1;
        } else {
            self.consecutive_failures = 0;
        }

        let rate_reached = self.failures >= config.failures && self.failures.saturating_mul(100) >= usize::from(config.rate).saturating_mul(self.requests);
        if self.consecutive_failures < config.failures && !rate_reached {
            return false;
        }
        if self.last_triggered_at.is_some_and(|triggered_at| now.duration_since(triggered_at) < config.window) {
            self.clear_samples();
            return false;
        }

        self.clear_samples();
        self.last_triggered_at = Some(now);
        true
    }

    fn record_sample(&mut self, failed: bool, now: Instant, window: Duration) {
        let bucket_width = (window / MAX_BUCKETS).max(Duration::from_nanos(1));
        if self.buckets.back().is_none_or(|bucket| now.duration_since(bucket.started_at) >= bucket_width) {
            self.buckets.push_back(FailureBucket {
                started_at: now,
                requests: 0,
                failures: 0,
            });
        }
        if let Some(bucket) = self.buckets.back_mut() {
            bucket.requests += 1;
            if failed {
                bucket.failures += 1;
            }
        }
    }

    fn prune(&mut self, now: Instant, window: Duration) {
        while self.buckets.front().is_some_and(|bucket| now.duration_since(bucket.started_at) >= window) {
            if let Some(bucket) = self.buckets.pop_front() {
                self.requests -= bucket.requests;
                self.failures -= bucket.failures;
            }
        }
    }

    pub(super) fn clear_samples(&mut self) {
        self.buckets.clear();
        self.requests = 0;
        self.failures = 0;
        self.consecutive_failures = 0;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn trigger_config(failures: usize) -> FailureTriggerConfig {
        FailureTriggerConfig {
            failures,
            rate: 50,
            window: Duration::from_secs(60),
        }
    }

    #[test]
    fn test_record() {
        let now = Instant::now();

        let mut tracker = FailureTracker::default();
        let config = trigger_config(15);
        for _ in 0..14 {
            assert!(!tracker.record(true, now, &config));
        }
        assert!(tracker.record(true, now, &config));

        let mut tracker = FailureTracker::default();
        for index in 0..29 {
            assert!(!tracker.record(index % 2 == 1, now, &config));
        }
        assert!(tracker.record(true, now, &config));

        let mut tracker = FailureTracker::default();
        let config = trigger_config(3);
        for failed in [true, false, true, false] {
            assert!(!tracker.record(failed, now, &config));
        }
        assert!(!tracker.record(true, now + Duration::from_secs(61), &config));

        let mut tracker = FailureTracker::default();
        for _ in 0..2 {
            assert!(!tracker.record(true, now, &config));
        }
        assert!(tracker.record(true, now, &config));
        for _ in 0..3 {
            assert!(!tracker.record(true, now + Duration::from_secs(30), &config));
        }
        for _ in 0..2 {
            assert!(!tracker.record(true, now + Duration::from_secs(61), &config));
        }
        assert!(tracker.record(true, now + Duration::from_secs(61), &config));

        let mut tracker = FailureTracker::default();
        let config = trigger_config(usize::MAX);
        for _ in 0..100_000 {
            assert!(!tracker.record(false, now, &config));
        }
        assert_eq!(tracker.buckets.len(), 1);
        tracker.clear_samples();
        assert!(tracker.buckets.is_empty());
    }
}
