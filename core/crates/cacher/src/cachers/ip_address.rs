use std::error::Error;

use primitives::IPAddressInfo;

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn ip_address(&self, ip_address: &str) -> Result<Option<IPAddressInfo>, Box<dyn Error + Send + Sync>> {
        self.get(CacheKey::FiatIpCheck(ip_address)).await
    }

    pub async fn add_ip_address(&self, ip_address: &str, info: &IPAddressInfo) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::FiatIpCheck(ip_address), info).await
    }
}
