use std::error::Error;

use async_trait::async_trait;
use cacher::RateLimiter;
use config_keys::{RateLimit, RateLimitKey};

#[async_trait]
pub trait RateLimits: Send + Sync {
    async fn consume(&self, key: RateLimitKey, scope: &str, limit: RateLimit) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl RateLimits for RateLimiter {
    async fn consume(&self, key: RateLimitKey, scope: &str, limit: RateLimit) -> Result<bool, Box<dyn Error + Send + Sync>> {
        RateLimiter::consume(self, key, scope, limit).await
    }
}
