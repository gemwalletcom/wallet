use std::error::Error;

use primitives::IpCheckResult;

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn ip_check(&self, ip_address: &str) -> Result<Option<IpCheckResult>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::ReferralIpCheck(ip_address)).await
    }

    pub async fn add_ip_check(&self, ip_address: &str, result: &IpCheckResult) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::ReferralIpCheck(ip_address), result).await
    }
}
