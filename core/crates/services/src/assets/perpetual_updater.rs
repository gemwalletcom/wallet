use std::error::Error;
use std::sync::Arc;

use chain_providers::ChainProviders;
use gem_tracing::error_with_fields;
use primitives::{Chain, asset_score::AssetRank};
use storage::AssetUpdate;

use crate::assets::repository::Repository;

pub struct PerpetualUpdater {
    providers: Arc<ChainProviders>,
    repository: Arc<dyn Repository>,
}

impl PerpetualUpdater {
    pub(crate) fn new(providers: Arc<ChainProviders>, repository: Arc<dyn Repository>) -> Self {
        Self { providers, repository }
    }

    pub fn chains() -> &'static [Chain] {
        &[Chain::HyperCore]
    }

    pub async fn update_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let perpetuals_data = self.providers.get_perpetuals_data(chain).await?;

        let assets = perpetuals_data.iter().map(|x| x.asset.clone()).collect::<Vec<_>>();
        let perpetuals = perpetuals_data.into_iter().map(|data| data.perpetual).collect::<Vec<_>>();
        let count = perpetuals.len();
        let asset_updates = vec![
            AssetUpdate::Rank(AssetRank::Unknown.threshold()),
            AssetUpdate::IsEnabled(false),
            AssetUpdate::IsSwappable(false),
            AssetUpdate::IsBuyable(false),
            AssetUpdate::IsSellable(false),
        ];
        let perpetuals_update = self.repository.update_perpetuals(assets, asset_updates, perpetuals).await?;

        if let Err(error) = perpetuals_update {
            error_with_fields!("failed perpetuals update", &error, chain = chain.as_ref());
        }
        Ok(count)
    }
}
