use std::error::Error;

use chain_providers::ProviderFactory;
use gem_tracing::error_with_fields;
use primitives::{Chain, asset_score::AssetRank};
use settings::{Settings, service_user_agent};
use storage::{AssetUpdate, AssetsRepository, Database, DatabaseError, PerpetualsRepository};

pub struct PerpetualUpdater {
    settings: Settings,
    database: Database,
}

impl PerpetualUpdater {
    pub fn new(settings: Settings, database: Database) -> Self {
        Self { settings, database }
    }

    pub fn chains() -> &'static [Chain] {
        &[Chain::HyperCore]
    }

    pub async fn update_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let provider = ProviderFactory::new_from_settings_with_user_agent(chain, &self.settings, &service_user_agent("daemon", Some("perpetual_updater")));
        let perpetuals_data = provider.get_perpetuals_data().await?;

        let assets = perpetuals_data.iter().map(|x| x.asset.clone()).collect::<Vec<_>>();
        let asset_ids = assets.iter().map(|x| x.id.clone()).collect::<Vec<_>>();
        let perpetuals = perpetuals_data.into_iter().map(|data| data.perpetual).collect::<Vec<_>>();
        let count = perpetuals.len();

        let perpetuals_update = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                client.upsert_assets(assets)?;
                client.update_assets(
                    asset_ids,
                    vec![
                        AssetUpdate::Rank(AssetRank::Unknown.threshold()),
                        AssetUpdate::IsEnabled(false),
                        AssetUpdate::IsSwappable(false),
                        AssetUpdate::IsBuyable(false),
                        AssetUpdate::IsSellable(false),
                    ],
                )?;
                Ok(client.perpetuals_update(perpetuals))
            })
            .await?;

        if let Err(e) = perpetuals_update {
            error_with_fields!("failed perpetuals update", &e, chain = chain.as_ref());
        }
        Ok(count)
    }
}
