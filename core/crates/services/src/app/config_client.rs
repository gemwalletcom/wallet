use std::error::Error;

use primitives::{AssetBasic, ConfigResponse, ConfigVersions, FiatAssets, SwapConfig, SwapProvider};
use storage::{AssetFilter, AssetsRepository, Database, DatabaseError, ReleasesRepository};

#[derive(Clone)]
pub struct ConfigClient {
    database: Database,
}

impl ConfigClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_config(&self) -> Result<ConfigResponse, Box<dyn Error + Send + Sync>> {
        let (fiat_on_ramp_assets, fiat_off_ramp_assets, swap_assets, releases) = self
            .database
            .run(|client| -> Result<_, DatabaseError> {
                Ok((
                    client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::IsBuyable(true)])?,
                    client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::IsSellable(true)])?,
                    client.get_swap_assets()?,
                    client.get_releases()?,
                ))
            })
            .await?;

        let response = ConfigResponse {
            releases,
            versions: ConfigVersions {
                fiat_on_ramp_assets: Self::version(fiat_on_ramp_assets),
                fiat_off_ramp_assets: Self::version(fiat_off_ramp_assets),
                swap_assets: FiatAssets::version(&swap_assets) as i32,
            },
            swap: SwapConfig {
                enabled_providers: SwapProvider::all().iter().map(|provider| provider.as_ref().to_string()).collect(),
            },
        };
        Ok(response)
    }

    fn version(assets: Vec<AssetBasic>) -> i32 {
        FiatAssets::version(&assets.into_iter().map(|asset| asset.asset.id.to_string()).collect::<Vec<String>>()) as i32
    }
}
