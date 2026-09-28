use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use job_runner::{JobContext, JobError, JobSchedule, RunDecision};

use crate::Services;

struct CacherJobTracker {
    cacher: CacherClient,
    service: String,
}

impl CacherJobTracker {
    fn new(cacher: CacherClient, service: &str) -> Self {
        Self { cacher, service: service.to_string() }
    }

    fn job_key(&self, job_name: &str) -> String {
        format!("{}:{}", self.service, job_name)
    }

    async fn get_last_success(&self, job_name: &str) -> Option<u64> {
        let key = self.job_key(job_name);
        let cache_key = CacheKey::JobStatus(&key);
        self.cacher.get_value(&cache_key.key()).await.ok()
    }
}

#[async_trait]
impl JobSchedule for CacherJobTracker {
    async fn evaluate(&self, job_name: &str, interval: Duration, now: SystemTime) -> Result<RunDecision, JobError> {
        let last_success_at = self.get_last_success(job_name).await;
        let ctx = JobContext { last_success_at };

        if let Some(last_success) = last_success_at {
            let last_success_time = UNIX_EPOCH + Duration::from_secs(last_success);
            let elapsed = now.duration_since(last_success_time).unwrap_or_default();
            if elapsed < interval {
                return Ok(RunDecision::Wait(interval - elapsed));
            }
        }
        Ok(RunDecision::Run(ctx))
    }

    async fn mark_success(&self, job_name: &str, timestamp: SystemTime) -> Result<(), JobError> {
        let seconds = timestamp.duration_since(UNIX_EPOCH).map_err(|err| Box::new(err) as JobError)?.as_secs();
        let key = self.job_key(job_name);
        let cache_key = CacheKey::JobStatus(&key);
        self.cacher.set_cached(cache_key, &seconds).await
    }
}

impl Services {
    pub async fn job_schedule(&self, service: &str) -> Result<Arc<dyn JobSchedule>, Box<dyn Error + Send + Sync>> {
        Ok(Arc::new(CacherJobTracker::new(self.cacher().await?, service)))
    }
}
