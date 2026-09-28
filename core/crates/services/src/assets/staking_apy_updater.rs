use std::error::Error;
use std::sync::Arc;

use chain_providers::ChainProviders;
use primitives::Chain;
use storage::{AssetUpdate, AssetsRepository, Database};

pub struct StakeApyUpdater {
    chain_providers: Arc<ChainProviders>,
    database: Database,
}

impl StakeApyUpdater {
    pub fn new(chain_providers: Arc<ChainProviders>, database: Database) -> Self {
        Self { chain_providers, database }
    }

    pub async fn update_chain(&self, chain: Chain) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let apy = self.chain_providers.get_staking_apy(chain).await?;
        let rounded = (apy * 100.0).round() / 100.0;
        self.database.run(move |client| client.update_assets(vec![chain.as_asset_id()], vec![AssetUpdate::StakingApr(Some(rounded))])).await?;
        Ok(rounded)
    }
}
