use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::SubscriptionAddressCacher;
use config_keys::ConfigKey;
use gem_tracing::warn_with_fields;
use primitives::{Chain, DeviceSubscription};

use crate::ConfigCacher;

pub(crate) mod repository;

use repository::Repository;

pub(crate) struct SubscriptionLookup {
    repository: Arc<dyn Repository>,
    cacher: Arc<dyn SubscriptionAddressCacher>,
    config: Arc<ConfigCacher>,
}

impl SubscriptionLookup {
    pub(crate) fn new(repository: Arc<dyn Repository>, cacher: Arc<dyn SubscriptionAddressCacher>, config: Arc<ConfigCacher>) -> Self {
        Self { repository, cacher, config }
    }

    pub(crate) async fn get(&self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, Box<dyn Error + Send + Sync>> {
        let addresses = addresses.into_iter().collect::<HashSet<_>>();
        if addresses.is_empty() {
            return Ok(vec![]);
        }

        let ttl = match self.cache_duration().await {
            Ok(Some(ttl)) => ttl,
            Ok(None) => return Ok(self.repository.subscriptions_for_addresses(chain, addresses.into_iter().collect()).await?),
            Err(error) => {
                warn_with_fields!("subscription address cache duration unavailable", error = &error);
                return Ok(self.repository.subscriptions_for_addresses(chain, addresses.into_iter().collect()).await?);
            }
        };
        let cache_addresses = addresses.iter().cloned().collect::<Vec<_>>();
        let addresses = match self.cacher.unsubscribed_addresses(chain, &cache_addresses).await {
            Ok(unsubscribed) => addresses.difference(&unsubscribed).cloned().collect::<Vec<_>>(),
            Err(error) => {
                warn_with_fields!("subscription address cache read failed", error = error.as_ref());
                addresses.into_iter().collect()
            }
        };
        if addresses.is_empty() {
            return Ok(vec![]);
        }

        let subscriptions = self.repository.subscriptions_for_addresses(chain, addresses.clone()).await?;
        let subscribed = subscriptions.iter().map(|subscription| subscription.address.as_str()).collect::<HashSet<_>>();
        let unsubscribed = addresses.into_iter().filter(|address| !subscribed.contains(address.as_str())).collect::<Vec<_>>();
        if !unsubscribed.is_empty()
            && let Err(error) = self.cacher.set_unsubscribed_addresses(chain, &unsubscribed, ttl).await
        {
            warn_with_fields!("subscription address cache write failed", error = error.as_ref());
        }
        Ok(subscriptions)
    }

    pub(crate) async fn cache_subscribed(&self, addresses: &[(Chain, String)]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if addresses.is_empty() {
            return Ok(());
        }
        if let Some(ttl) = self.cache_duration().await? {
            self.cacher.set_subscribed_addresses(addresses, ttl).await?;
        }
        Ok(())
    }

    async fn cache_duration(&self) -> Result<Option<Duration>, storage::DatabaseError> {
        let ttl = self.config.get_duration(ConfigKey::SubscriptionAddressStatusCacheDuration).await?;
        Ok((ttl.as_secs() > 0).then_some(ttl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{MemoryConfigRepository, MemorySubscriptionAddressCacher, MemorySubscriptionRepository};

    #[tokio::test]
    async fn test_subscribed_marker_wins_over_late_unsubscribed_marker() {
        let repository = Arc::new(MemorySubscriptionRepository::default());
        let cacher = Arc::new(MemorySubscriptionAddressCacher::default());
        let lookup = SubscriptionLookup::new(repository.clone(), cacher.clone(), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))));
        let chain = Chain::Ethereum;
        let address = "0x123".to_string();

        assert!(lookup.get(chain, vec![address.clone()]).await.unwrap().is_empty());
        assert!(lookup.get(chain, vec![address.clone()]).await.unwrap().is_empty());
        assert_eq!(repository.reads(), 1);

        repository.set_subscriptions(vec![DeviceSubscription {
            chain,
            address: address.clone(),
            ..DeviceSubscription::mock()
        }]);
        lookup.cache_subscribed(&[(chain, address.clone())]).await.unwrap();
        cacher.set_unsubscribed_addresses(chain, std::slice::from_ref(&address), std::time::Duration::from_secs(900)).await.unwrap();
        let subscriptions = lookup.get(chain, vec![address.clone()]).await.unwrap();
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].address, address);
        assert_eq!(repository.reads(), 2);
    }

    #[tokio::test]
    async fn test_get_always_reads_current_subscriptions() {
        let repository = Arc::new(MemorySubscriptionRepository::default());
        let lookup = SubscriptionLookup::new(repository.clone(), Arc::new(MemorySubscriptionAddressCacher::default()), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))));
        let chain = Chain::Ethereum;
        let address = "0x123".to_string();
        let mut subscription = DeviceSubscription {
            chain,
            address: address.clone(),
            ..DeviceSubscription::mock()
        };
        repository.set_subscriptions(vec![subscription.clone()]);

        assert_eq!(lookup.get(chain, vec![address.clone()]).await.unwrap()[0].device.token, subscription.device.token);

        subscription.device.token = "updated-token".to_string();
        repository.set_subscriptions(vec![subscription]);
        assert_eq!(lookup.get(chain, vec![address]).await.unwrap()[0].device.token, "updated-token");
        assert_eq!(repository.reads(), 2);
    }

    #[tokio::test]
    async fn test_get_falls_back_to_repository_when_cache_is_unavailable() {
        let repository = Arc::new(MemorySubscriptionRepository::default());
        let lookup = SubscriptionLookup::new(
            repository.clone(),
            Arc::new(MemorySubscriptionAddressCacher::unavailable()),
            Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))),
        );
        let chain = Chain::Ethereum;
        let address = "0x123".to_string();
        repository.set_subscriptions(vec![DeviceSubscription {
            chain,
            address: address.clone(),
            ..DeviceSubscription::mock()
        }]);

        assert_eq!(lookup.get(chain, vec![address.clone()]).await.unwrap()[0].address, address);
        assert_eq!(repository.reads(), 1);
    }

    #[tokio::test]
    async fn test_get_falls_back_to_repository_when_config_is_unavailable() {
        let repository = Arc::new(MemorySubscriptionRepository::default());
        let lookup = SubscriptionLookup::new(
            repository.clone(),
            Arc::new(MemorySubscriptionAddressCacher::unavailable()),
            Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::unavailable()))),
        );
        let chain = Chain::Ethereum;
        let address = "0x123".to_string();
        repository.set_subscriptions(vec![DeviceSubscription {
            chain,
            address: address.clone(),
            ..DeviceSubscription::mock()
        }]);

        assert_eq!(lookup.get(chain, vec![address.clone()]).await.unwrap()[0].address, address);
        assert_eq!(repository.reads(), 1);
    }

    #[tokio::test]
    async fn test_cache_subscribed_propagates_cache_failure() {
        let lookup = SubscriptionLookup::new(
            Arc::new(MemorySubscriptionRepository::default()),
            Arc::new(MemorySubscriptionAddressCacher::unavailable()),
            Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))),
        );

        assert!(lookup.cache_subscribed(&[(Chain::Ethereum, "0x123".to_string())]).await.is_err());
    }

    #[tokio::test]
    async fn test_cache_subscribed_propagates_config_failure() {
        let lookup = SubscriptionLookup::new(
            Arc::new(MemorySubscriptionRepository::default()),
            Arc::new(MemorySubscriptionAddressCacher::default()),
            Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::unavailable()))),
        );

        assert!(lookup.cache_subscribed(&[(Chain::Ethereum, "0x123".to_string())]).await.is_err());
    }

    #[tokio::test]
    async fn test_cache_subscribed_skips_cache_when_disabled() {
        let config = MemoryConfigRepository::new().with_value(ConfigKey::SubscriptionAddressStatusCacheDuration.as_ref(), "0s");
        let lookup = SubscriptionLookup::new(
            Arc::new(MemorySubscriptionRepository::default()),
            Arc::new(MemorySubscriptionAddressCacher::unavailable()),
            Arc::new(ConfigCacher::new(Arc::new(config))),
        );

        lookup.cache_subscribed(&[(Chain::Ethereum, "0x123".to_string())]).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_filters_chain_and_addresses() {
        let repository = Arc::new(MemorySubscriptionRepository::default());
        let lookup = SubscriptionLookup::new(repository.clone(), Arc::new(MemorySubscriptionAddressCacher::default()), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))));
        let address = "0x123".to_string();
        repository.set_subscriptions(vec![
            DeviceSubscription {
                chain: Chain::Ethereum,
                address: address.clone(),
                ..DeviceSubscription::mock()
            },
            DeviceSubscription {
                chain: Chain::Bitcoin,
                address: address.clone(),
                ..DeviceSubscription::mock()
            },
            DeviceSubscription {
                chain: Chain::Ethereum,
                address: "0x456".to_string(),
                ..DeviceSubscription::mock()
            },
        ]);

        let subscriptions = lookup.get(Chain::Ethereum, vec![address.clone()]).await.unwrap();
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].chain, Chain::Ethereum);
        assert_eq!(subscriptions[0].address, address);
    }
}
