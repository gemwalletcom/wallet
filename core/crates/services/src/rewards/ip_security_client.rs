use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::try_in_order;
use rewards::{IpCheckProvider, IpCheckResult};

#[async_trait]
pub trait IpCheckStore: Send + Sync {
    async fn ip_check(&self, ip_address: &str) -> Result<Option<IpCheckResult>, Box<dyn Error + Send + Sync>>;
    async fn add_ip_check(&self, ip_address: &str, result: &IpCheckResult) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl IpCheckStore for CacherClient {
    async fn ip_check(&self, ip_address: &str) -> Result<Option<IpCheckResult>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(CacheKey::ReferralIpCheck(ip_address)).await
    }

    async fn add_ip_check(&self, ip_address: &str, result: &IpCheckResult) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::ReferralIpCheck(ip_address), result).await
    }
}

pub struct IpSecurityClient {
    providers: Vec<Arc<dyn IpCheckProvider>>,
    checks: Arc<dyn IpCheckStore>,
}

impl IpSecurityClient {
    pub fn new(providers: Vec<Arc<dyn IpCheckProvider>>, checks: Arc<dyn IpCheckStore>) -> Self {
        Self { providers, checks }
    }

    pub async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        if let Ok(Some(result)) = self.checks.ip_check(ip_address).await {
            return Ok(result);
        }
        let result = self.check_ip_with_fallback(ip_address).await?;
        self.checks.add_ip_check(ip_address, &result).await?;
        Ok(result)
    }

    async fn check_ip_with_fallback(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let operations = self.providers.iter().map(|provider| provider.check_ip(ip_address)).collect::<Vec<_>>();
        match try_in_order(operations).await? {
            Some(result) => Ok(result),
            None => Err("No IP check providers configured".into()),
        }
    }
}
