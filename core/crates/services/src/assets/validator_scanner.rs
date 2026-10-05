use crate::StaticAssetsClient;
use chain_providers::ChainProviders;
use primitives::{Chain, StakeValidator};
use std::error::Error;
use std::sync::Arc;

use crate::assets::repository::Repository;

pub struct ValidatorScanner {
    chain_providers: Arc<ChainProviders>,
    static_assets_client: StaticAssetsClient,
    repository: Arc<dyn Repository>,
}

impl ValidatorScanner {
    pub(crate) fn new(chain_providers: Arc<ChainProviders>, static_assets_client: StaticAssetsClient, repository: Arc<dyn Repository>) -> Self {
        Self {
            chain_providers,
            static_assets_client,
            repository,
        }
    }

    pub async fn update_validators_for_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let validators = self.chain_providers.get_validators(chain).await?;
        let addresses: Vec<_> = validators.into_iter().filter_map(|v| v.as_scan_address(chain)).collect();
        let count = addresses.len();
        self.repository.add_scan_addresses(addresses).await?;
        Ok(count)
    }

    pub async fn update_validators_from_static_assets_for_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let static_validators = self.static_assets_client.get_validators(chain).await?;
        let addresses: Vec<_> = static_validators.into_iter().map(|v| StakeValidator::new(v.id, v.name)).filter_map(|v| v.as_scan_address(chain)).collect();
        let count = addresses.len();
        self.repository.add_scan_addresses(addresses).await?;
        Ok(count)
    }
}
