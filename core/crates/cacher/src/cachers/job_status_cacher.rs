use std::error::Error;

use async_trait::async_trait;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait JobStatusCacher: Send + Sync {
    async fn last_success(&self, job: &str) -> Result<Option<u64>, Box<dyn Error + Send + Sync>>;
    async fn set_last_success(&self, job: &str, timestamp: u64) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl JobStatusCacher for CacherClient {
    async fn last_success(&self, job: &str) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        self.get_value_optional(&CacheKey::JobStatus(job).key()).await
    }

    async fn set_last_success(&self, job: &str, timestamp: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::JobStatus(job), &timestamp).await
    }
}
