use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{PerpetualAddressCacher, PerpetualAddressTier};
use chain_providers::ChainProviders;
use gem_tracing::info_with_fields;
use primitives::Chain;

use super::repository::Repository;
use crate::subscriptions::SubscriptionLookup;

pub struct PerpetualAddressRefresher {
    providers: Arc<ChainProviders>,
    repository: Arc<dyn Repository>,
    subscription_lookup: Arc<SubscriptionLookup>,
    addresses: Arc<dyn PerpetualAddressCacher>,
}

impl PerpetualAddressRefresher {
    pub(crate) fn new(providers: Arc<ChainProviders>, repository: Arc<dyn Repository>, subscription_lookup: Arc<SubscriptionLookup>, addresses: Arc<dyn PerpetualAddressCacher>) -> Self {
        Self {
            providers,
            repository,
            subscription_lookup,
            addresses,
        }
    }

    pub async fn update(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let addresses = self.providers.get_perpetual_addresses(chain).await?;
        let candidate_addresses: Vec<String> = addresses.iter().map(|address| address.address.clone()).collect();
        self.repository.set_perpetual_addresses(addresses).await?;
        let candidate_count = candidate_addresses.len();

        let tracked_addresses: Vec<String> = self.subscription_lookup.get(chain, candidate_addresses).await?.into_iter().map(|s| s.address).collect::<HashSet<_>>().into_iter().collect();

        self.addresses.set_addresses(chain, PerpetualAddressTier::Tracked, &tracked_addresses).await?;

        info_with_fields!("perpetual_refresher", chain = chain.as_ref(), candidates = candidate_count, tracked = tracked_addresses.len());

        Ok(tracked_addresses.len())
    }
}
