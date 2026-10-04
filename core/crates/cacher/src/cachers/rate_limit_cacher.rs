use std::error::Error;

use async_trait::async_trait;
use config_keys::{RateLimit, RateLimitKey, RateLimitWindow};

use crate::{CacheKey, CacherClient};

pub const GLOBAL_RATE_LIMIT_SCOPE: &str = "global";

#[async_trait]
pub trait RateLimitCacher: Send + Sync {
    async fn consume(&self, key: RateLimitKey, scope: &str, limit: RateLimit) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl RateLimitCacher for CacherClient {
    async fn consume(&self, key: RateLimitKey, scope: &str, limit: RateLimit) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut allowed = true;
        for window in RateLimitWindow::ALL {
            allowed &= self.increment_cached(CacheKey::RateLimit(key, scope, window)).await? <= limit.get(window);
        }
        Ok(allowed)
    }
}
