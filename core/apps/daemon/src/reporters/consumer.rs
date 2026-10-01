use std::sync::Arc;

use async_trait::async_trait;
use streamer::{ConsumerStatus, ConsumerStatusReporter};

use crate::metrics::consumer::ConsumerMetrics;

pub struct ConsumerReporter {
    metrics: Arc<ConsumerMetrics>,
}

impl ConsumerReporter {
    pub fn new(metrics: Arc<ConsumerMetrics>) -> Self {
        Self { metrics }
    }
}

#[async_trait]
impl ConsumerStatusReporter for ConsumerReporter {
    async fn report(&self, name: &str, status: ConsumerStatus) {
        match status {
            ConsumerStatus::Started { queue_wait_seconds } => self.metrics.record_started(name, queue_wait_seconds),
            ConsumerStatus::Success { duration_milliseconds } => self.metrics.record_success(name, duration_milliseconds),
            ConsumerStatus::Skipped { duration_milliseconds } => self.metrics.record_skipped(name, duration_milliseconds),
            ConsumerStatus::Error { duration_milliseconds } => self.metrics.record_error(name, duration_milliseconds),
        }
    }
}
