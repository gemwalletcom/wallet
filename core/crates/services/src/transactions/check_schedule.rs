use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use chrono::{DateTime, Utc};
use primitives::{JobConfiguration, TransactionId};

use crate::transactions::TransactionQueue;

#[async_trait]
pub trait CheckScheduleStore: Send + Sync {
    async fn next_checks(&self, queue: TransactionQueue, ids: &[String]) -> Result<Vec<Option<f64>>, Box<dyn Error + Send + Sync>>;
    async fn set_next_check(&self, queue: TransactionQueue, id: String, next_check_at: f64) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn remove_check(&self, queue: TransactionQueue, id: String) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl CheckScheduleStore for CacherClient {
    async fn next_checks(&self, queue: TransactionQueue, ids: &[String]) -> Result<Vec<Option<f64>>, Box<dyn Error + Send + Sync>> {
        self.sorted_set_scores(&schedule_key(queue).key(), ids).await
    }

    async fn set_next_check(&self, queue: TransactionQueue, id: String, next_check_at: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_sorted_set_cached(schedule_key(queue), &[(id, next_check_at)]).await?;
        Ok(())
    }

    async fn remove_check(&self, queue: TransactionQueue, id: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.remove_from_sorted_set_cached(schedule_key(queue), &[id]).await?;
        Ok(())
    }
}

fn schedule_key(queue: TransactionQueue) -> CacheKey<'static> {
    CacheKey::TransactionCheckSchedule(queue.into())
}

pub struct CheckSchedule {
    store: Arc<dyn CheckScheduleStore>,
    queue: TransactionQueue,
}

impl CheckSchedule {
    pub fn new(store: Arc<dyn CheckScheduleStore>, queue: TransactionQueue) -> Self {
        Self { store, queue }
    }

    pub async fn due(&self, ids: &[TransactionId], now: DateTime<Utc>) -> Result<HashSet<TransactionId>, Box<dyn Error + Send + Sync>> {
        let members = ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let scores = self.store.next_checks(self.queue, &members).await?;
        Ok(ids.iter().zip(scores).filter(|(_, next_check_at)| is_due(*next_check_at, now)).map(|(id, _)| id.clone()).collect())
    }

    pub async fn schedule_next(&self, id: &TransactionId, configuration: &JobConfiguration, elapsed: Duration, now: DateTime<Utc>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let next_check_at = now + configuration.next_check_interval(elapsed);
        self.store.set_next_check(self.queue, id.to_string(), timestamp(next_check_at)).await
    }

    pub async fn remove(&self, id: &TransactionId) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.store.remove_check(self.queue, id.to_string()).await
    }
}

fn is_due(next_check_at: Option<f64>, now: DateTime<Utc>) -> bool {
    next_check_at.is_none_or(|next_check_at| next_check_at <= timestamp(now))
}

fn timestamp(time: DateTime<Utc>) -> f64 {
    time.timestamp_millis() as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_due() {
        let now = Utc::now();

        assert!(is_due(None, now));
        assert!(is_due(Some(timestamp(now)), now));
        assert!(!is_due(Some(timestamp(now) + 1.0), now));
    }
}
