use std::error::Error;

use async_trait::async_trait;
use primitives::IpCheckResult;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait IpCheckCacher: Send + Sync {
    async fn ip_check(&self, ip_address: &str) -> Result<Option<IpCheckResult>, Box<dyn Error + Send + Sync>>;
    async fn add_ip_check(&self, ip_address: &str, result: &IpCheckResult) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl IpCheckCacher for CacherClient {
    async fn ip_check(&self, ip_address: &str) -> Result<Option<IpCheckResult>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::ReferralIpCheck(ip_address)).await
    }

    async fn add_ip_check(&self, ip_address: &str, result: &IpCheckResult) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::ReferralIpCheck(ip_address), result).await
    }
}
