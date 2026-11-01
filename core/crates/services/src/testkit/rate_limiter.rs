use std::{collections::HashMap, error::Error, sync::Mutex};

use async_trait::async_trait;
use cacher::RateLimitCacher;
use config_keys::{RateLimit, RateLimitKey, RateLimitWindow};

#[derive(Default)]
pub(crate) struct MemoryRateLimiter {
    counts: Mutex<HashMap<String, i64>>,
}

impl MemoryRateLimiter {
    pub(crate) fn consumed(&self, scope: &str) -> i64 {
        self.counts.lock().unwrap().get(scope).copied().unwrap_or_default()
    }
}

#[async_trait]
impl RateLimitCacher for MemoryRateLimiter {
    async fn consume(&self, _key: RateLimitKey, scope: &str, limit: RateLimit) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut counts = self.counts.lock().unwrap();
        let count = counts.entry(scope.to_string()).or_default();
        *count += 1;
        Ok(*count <= limit.get(RateLimitWindow::Minute))
    }
}
