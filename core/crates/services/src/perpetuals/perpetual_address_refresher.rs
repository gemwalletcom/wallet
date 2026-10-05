use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::{PerpetualAddressCacher, PerpetualAddressTier};
use chain_providers::ChainProviders;
use gem_tracing::info_with_fields;
use primitives::Chain;

use crate::subscriptions::SubscriptionLookup;

pub struct PerpetualAddressRefresher {
    providers: Arc<ChainProviders>,
    subscription_lookup: Arc<SubscriptionLookup>,
    addresses: Arc<dyn PerpetualAddressCacher>,
}

impl PerpetualAddressRefresher {
    pub(crate) fn new(providers: Arc<ChainProviders>, subscription_lookup: Arc<SubscriptionLookup>, addresses: Arc<dyn PerpetualAddressCacher>) -> Self {
        Self { providers, subscription_lookup, addresses }
    }

    pub async fn update(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let candidate_addresses = self.providers.get_perpetual_referral_addresses(chain).await?;
        let candidate_count = candidate_addresses.len();

        let tracked_addresses: Vec<String> = if candidate_addresses.is_empty() {
            vec![]
        } else {
            self.subscription_lookup.get(chain, candidate_addresses).await?.into_iter().map(|s| s.address).collect::<HashSet<_>>().into_iter().collect()
        };

        self.addresses.set_addresses(chain, PerpetualAddressTier::Tracked, &tracked_addresses).await?;

        info_with_fields!("perpetual_refresher", chain = chain.as_ref(), candidates = candidate_count, tracked = tracked_addresses.len());

        Ok(tracked_addresses.len())
    }
}
