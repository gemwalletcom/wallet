use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::IpAddressCacher;
use fiat::IpAddressProvider;
use primitives::IPAddressInfo;

pub(crate) struct CachedIpAddressProvider {
    addresses: Arc<dyn IpAddressCacher>,
    provider: Arc<dyn IpAddressProvider>,
}

impl CachedIpAddressProvider {
    pub(crate) fn new(addresses: Arc<dyn IpAddressCacher>, provider: Arc<dyn IpAddressProvider>) -> Self {
        Self { addresses, provider }
    }
}

#[async_trait]
impl IpAddressProvider for CachedIpAddressProvider {
    async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        if let Ok(Some(info)) = self.addresses.ip_address(ip_address).await {
            return Ok(info);
        }
        let info = self.provider.get_ip_address(ip_address).await?;
        self.addresses.add_ip_address(ip_address, &info).await?;
        Ok(info)
    }
}
