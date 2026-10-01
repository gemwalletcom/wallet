use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use cacher::{CacheKey, CacherClient};
use chrono::{DateTime, Utc};
use primitives::{JobConfiguration, TransactionId};

use crate::transactions::TransactionQueue;

pub struct CheckSchedule {
    cacher: CacherClient,
    queue: TransactionQueue,
}

impl CheckSchedule {
    pub fn new(cacher: CacherClient, queue: TransactionQueue) -> Self {
        Self { cacher, queue }
    }

    pub async fn due(&self, ids: &[TransactionId], now: DateTime<Utc>) -> Result<HashSet<TransactionId>, Box<dyn Error + Send + Sync>> {
        let members = ids.iter().map(ToString::to_string).collect::<Vec<_>>();
        let scores = self.cacher.sorted_set_scores(&self.key().key(), &members).await?;
        Ok(ids.iter().zip(scores).filter(|(_, next_check_at)| is_due(*next_check_at, now)).map(|(id, _)| id.clone()).collect())
    }

    pub async fn schedule_next(&self, id: &TransactionId, configuration: &JobConfiguration, elapsed: Duration, now: DateTime<Utc>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let next_check_at = now + configuration.next_check_interval(elapsed);
        self.cacher.add_to_sorted_set_cached(self.key(), &[(id.to_string(), timestamp(next_check_at))]).await?;
        Ok(())
    }

    pub async fn remove(&self, id: &TransactionId) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher.remove_from_sorted_set_cached(self.key(), &[id.to_string()]).await?;
        Ok(())
    }

    fn key(&self) -> CacheKey<'static> {
        CacheKey::TransactionCheckSchedule(self.queue.into())
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
