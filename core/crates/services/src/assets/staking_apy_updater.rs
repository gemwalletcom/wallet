use std::error::Error;
use std::sync::Arc;

use chain_providers::ChainProviders;
use primitives::Chain;
use storage::{AssetFilter, AssetUpdate};

use crate::assets::repository::Repository;

pub struct StakeApyUpdater {
    chain_providers: Arc<ChainProviders>,
    repository: Arc<dyn Repository>,
}

impl StakeApyUpdater {
    pub(crate) fn new(chain_providers: Arc<ChainProviders>, repository: Arc<dyn Repository>) -> Self {
        Self { chain_providers, repository }
    }

    pub async fn update_chain(&self, chain: Chain) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let apy = self.chain_providers.get_staking_apy(chain).await?;
        let rounded = (apy * 100.0).round() / 100.0;
        self.repository
            .update_assets(vec![AssetFilter::Ids(vec![chain.as_asset_id().to_string()])], vec![AssetUpdate::StakingApr(Some(rounded))])
            .await?;
        Ok(rounded)
    }
}
