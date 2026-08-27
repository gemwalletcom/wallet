use std::error::Error;

use primitives::{RateLimit, RateLimitKey, RateLimitWindow};

use crate::{CacheKey, CacherClient};

pub const GLOBAL_RATE_LIMIT_SCOPE: &str = "global";

#[derive(Clone)]
pub struct RateLimiter {
    cacher: CacherClient,
}

impl RateLimiter {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }

    pub async fn consume(&self, key: RateLimitKey, scope: &str, limit: RateLimit) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut allowed = true;
        for window in RateLimitWindow::ALL {
            allowed &= self.cacher.increment_cached(CacheKey::RateLimit(key, scope, window)).await? <= limit.get(window);
        }
        Ok(allowed)
    }
}
