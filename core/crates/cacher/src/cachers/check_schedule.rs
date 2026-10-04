use std::error::Error;

use async_trait::async_trait;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait CheckScheduleCacher: Send + Sync {
    async fn next_checks(&self, queue: &str, ids: &[String]) -> Result<Vec<Option<f64>>, Box<dyn Error + Send + Sync>>;
    async fn set_next_check(&self, queue: &str, id: String, next_check_at: f64) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn remove_check(&self, queue: &str, id: String) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl CheckScheduleCacher for CacherClient {
    async fn next_checks(&self, queue: &str, ids: &[String]) -> Result<Vec<Option<f64>>, Box<dyn Error + Send + Sync>> {
        self.sorted_set_scores(CacheKey::TransactionCheckSchedule(queue), ids).await
    }

    async fn set_next_check(&self, queue: &str, id: String, next_check_at: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_sorted_set(CacheKey::TransactionCheckSchedule(queue), &[(id, next_check_at)]).await?;
        Ok(())
    }

    async fn remove_check(&self, queue: &str, id: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.remove_from_sorted_set(CacheKey::TransactionCheckSchedule(queue), &[id]).await?;
        Ok(())
    }
}
