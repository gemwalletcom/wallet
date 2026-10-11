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
        let asset_id = chain.as_asset_id().to_string();
        self.repository.update_assets(vec![AssetFilter::Ids(vec![asset_id.clone()])], vec![AssetUpdate::StakingApr(Some(rounded))]).await?;
        self.repository
            .update_assets(vec![AssetFilter::Ids(vec![asset_id]), AssetFilter::IsStakeable(false)], vec![AssetUpdate::IsStakeable(true)])
            .await?;
        Ok(rounded)
    }
}
