use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use fiat::{IPAddressInfo, IpAddressProvider};

pub(crate) struct CachedIpAddressProvider {
    cacher: CacherClient,
    provider: Arc<dyn IpAddressProvider>,
}

impl CachedIpAddressProvider {
    pub(crate) fn new(cacher: CacherClient, provider: Arc<dyn IpAddressProvider>) -> Self {
        Self { cacher, provider }
    }
}

#[async_trait]
impl IpAddressProvider for CachedIpAddressProvider {
    async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        self.cacher.get_or_set_cached(CacheKey::FiatIpCheck(ip_address), || self.provider.get_ip_address(ip_address)).await
    }
}
