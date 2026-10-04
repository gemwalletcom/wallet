use std::error::Error;

use async_trait::async_trait;
use primitives::IPAddressInfo;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait IpAddressCacher: Send + Sync {
    async fn ip_address(&self, ip_address: &str) -> Result<Option<IPAddressInfo>, Box<dyn Error + Send + Sync>>;
    async fn add_ip_address(&self, ip_address: &str, info: &IPAddressInfo) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl IpAddressCacher for CacherClient {
    async fn ip_address(&self, ip_address: &str) -> Result<Option<IPAddressInfo>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::FiatIpCheck(ip_address)).await
    }

    async fn add_ip_address(&self, ip_address: &str, info: &IPAddressInfo) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::FiatIpCheck(ip_address), info).await
    }
}
