use std::error::Error;

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn last_success(&self, job: &str) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::JobStatus(job)).await
    }

    pub async fn set_last_success(&self, job: &str, timestamp: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::JobStatus(job), &timestamp).await
    }
}
