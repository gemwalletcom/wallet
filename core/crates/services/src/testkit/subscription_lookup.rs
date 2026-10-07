use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cacher::SubscriptionAddressCacher;
use primitives::{Chain, DeviceSubscription};
use storage::DatabaseError;

use crate::subscriptions::repository::Repository;

#[derive(Default)]
pub(crate) struct MemorySubscriptionRepository {
    reads: Mutex<usize>,
    subscriptions: Mutex<Vec<DeviceSubscription>>,
}

impl MemorySubscriptionRepository {
    pub(crate) fn reads(&self) -> usize {
        *self.reads.lock().unwrap()
    }

    pub(crate) fn set_subscriptions(&self, subscriptions: Vec<DeviceSubscription>) {
        *self.subscriptions.lock().unwrap() = subscriptions;
    }
}

#[async_trait]
impl Repository for MemorySubscriptionRepository {
    async fn subscriptions_for_addresses(&self, chain: Chain, addresses: Vec<String>) -> Result<Vec<DeviceSubscription>, DatabaseError> {
        *self.reads.lock().unwrap() += 1;
        let addresses = addresses.into_iter().collect::<HashSet<_>>();
        Ok(self
            .subscriptions
            .lock()
            .unwrap()
            .iter()
            .filter(|subscription| subscription.chain == chain && addresses.contains(&subscription.address))
            .cloned()
            .collect())
    }
}

#[derive(Default)]
pub(crate) struct MemorySubscriptionAddressCacher {
    statuses: Mutex<HashMap<(Chain, String), bool>>,
    unavailable: bool,
}

impl MemorySubscriptionAddressCacher {
    pub(crate) fn unavailable() -> Self {
        Self {
            statuses: Mutex::new(HashMap::new()),
            unavailable: true,
        }
    }

    fn check_available(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.unavailable { Err(std::io::Error::other("subscription cache unavailable").into()) } else { Ok(()) }
    }
}

#[async_trait]
impl SubscriptionAddressCacher for MemorySubscriptionAddressCacher {
    async fn unsubscribed_addresses(&self, chain: Chain, addresses: &[String]) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
        self.check_available()?;
        let statuses = self.statuses.lock().unwrap();
        Ok(addresses.iter().filter(|address| statuses.get(&(chain, (*address).clone())) == Some(&false)).cloned().collect())
    }

    async fn set_unsubscribed_addresses(&self, chain: Chain, addresses: &[String], _ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.check_available()?;
        let mut statuses = self.statuses.lock().unwrap();
        for address in addresses {
            statuses.entry((chain, address.clone())).or_insert(false);
        }
        Ok(())
    }

    async fn set_subscribed_addresses(&self, addresses: &[(Chain, String)], _ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.check_available()?;
        let mut statuses = self.statuses.lock().unwrap();
        for (chain, address) in addresses {
            statuses.insert((*chain, address.clone()), true);
        }
        Ok(())
    }
}
