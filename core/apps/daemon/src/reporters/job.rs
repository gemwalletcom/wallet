use std::sync::Arc;

use async_trait::async_trait;
use job_runner::JobStatusReporter;

use crate::metrics::job::JobMetrics;

pub struct JobReporter {
    metrics: Arc<JobMetrics>,
}

impl JobReporter {
    pub fn new(metrics: Arc<JobMetrics>) -> Self {
        Self { metrics }
    }
}

#[async_trait]
impl JobStatusReporter for JobReporter {
    async fn restore(&self, name: &str, interval: u64, last_success_at: Option<u64>) {
        self.metrics.restore(name, interval, last_success_at);
    }

    async fn report(&self, name: &str, interval: u64, duration: u64, success: bool) {
        self.metrics.report(name, interval, duration, success);
    }
}
