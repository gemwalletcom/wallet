use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::Chain;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait SubscriptionAddressCacher: Send + Sync {
    async fn unsubscribed_addresses(&self, chain: Chain, addresses: &[String]) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>>;
    async fn set_unsubscribed_addresses(&self, chain: Chain, addresses: &[String], ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn set_subscribed_addresses(&self, addresses: &[(Chain, String)], ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl SubscriptionAddressCacher for CacherClient {
    async fn unsubscribed_addresses(&self, chain: Chain, addresses: &[String]) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
        let keys = addresses.iter().map(|address| CacheKey::SubscriptionAddressStatus(chain.as_ref(), address, 0)).collect::<Vec<_>>();
        let statuses = self.get_many_optional::<bool>(&keys).await?;
        Ok(addresses.iter().zip(statuses).filter(|(_, subscribed)| *subscribed == Some(false)).map(|(address, _)| address.clone()).collect())
    }

    async fn set_unsubscribed_addresses(&self, chain: Chain, addresses: &[String], ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = addresses.iter().map(|address| (CacheKey::SubscriptionAddressStatus(chain.as_ref(), address, ttl.as_secs()), &false)).collect::<Vec<_>>();
        self.set_many_if_absent(&entries).await?;
        Ok(())
    }

    async fn set_subscribed_addresses(&self, addresses: &[(Chain, String)], ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = addresses
            .iter()
            .map(|(chain, address)| (CacheKey::SubscriptionAddressStatus(chain.as_ref(), address, ttl.as_secs()), &true))
            .collect::<Vec<_>>();
        self.set_many(&entries).await?;
        Ok(())
    }
}
