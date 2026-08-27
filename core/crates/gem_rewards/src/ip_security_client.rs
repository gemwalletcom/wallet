use std::error::Error;
use std::sync::Arc;

use cacher::{CacheKey, CacherClient};
use primitives::try_in_order;

use crate::ip_check_provider::IpCheckProvider;
use crate::model::IpCheckResult;

pub struct IpSecurityClient {
    providers: Vec<Arc<dyn IpCheckProvider>>,
    cacher: CacherClient,
}

impl IpSecurityClient {
    pub fn new(providers: Vec<Arc<dyn IpCheckProvider>>, cacher: CacherClient) -> Self {
        Self { providers, cacher }
    }

    pub async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        self.cacher
            .get_or_set_cached(CacheKey::ReferralIpCheck(ip_address), || async { self.check_ip_with_fallback(ip_address).await })
            .await
    }

    async fn check_ip_with_fallback(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let operations = self.providers.iter().map(|provider| provider.check_ip(ip_address)).collect::<Vec<_>>();
        match try_in_order(operations).await? {
            Some(result) => Ok(result),
            None => Err("No IP check providers configured".into()),
        }
    }
}
