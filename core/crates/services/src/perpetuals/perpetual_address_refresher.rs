use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use chain_providers::ChainProviders;
use gem_tracing::info_with_fields;
use primitives::Chain;
use storage::{Database, WalletsRepository};

use super::{PerpetualAddressCacher, PerpetualAddressTier};

pub struct PerpetualAddressRefresher {
    providers: Arc<ChainProviders>,
    database: Database,
    addresses: Arc<dyn PerpetualAddressCacher>,
}

impl PerpetualAddressRefresher {
    pub fn new(providers: Arc<ChainProviders>, database: Database, addresses: Arc<dyn PerpetualAddressCacher>) -> Self {
        Self { providers, database, addresses }
    }

    pub async fn update(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let referred_addresses = self.providers.get_perpetual_referred_addresses(chain).await?;
        let referred_count = referred_addresses.len();

        let tracked_addresses: Vec<String> = if referred_addresses.is_empty() {
            vec![]
        } else {
            self.database
                .run(move |client| client.get_subscriptions_by_chain_addresses(chain, referred_addresses))
                .await?
                .into_iter()
                .map(|s| s.address)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };

        self.addresses.set_addresses(chain, PerpetualAddressTier::Tracked, &tracked_addresses).await?;

        info_with_fields!("perpetual_refresher", chain = chain.as_ref(), referred = referred_count, tracked = tracked_addresses.len());

        Ok(tracked_addresses.len())
    }
}
