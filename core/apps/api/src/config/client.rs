use primitives::{AssetBasic, ConfigResponse, ConfigVersions, FiatAssets, SwapConfig, SwapProvider};
use std::error::Error;
use storage::{AssetFilter, AssetsRepository, Database, ReleasesRepository};

#[derive(Clone)]
pub struct ConfigClient {
    database: Database,
}

impl ConfigClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn get_config(&self) -> Result<ConfigResponse, Box<dyn Error + Send + Sync>> {
        let fiat_on_ramp_assets = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::IsBuyable(true)])?;
        let fiat_off_ramp_assets = self
            .database
            .assets()?
            .get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::IsSellable(true)])?;
        let swap_assets = self.database.assets()?.get_swap_assets()?;
        let releases = self.database.releases()?.get_releases()?;

        let releases = releases.into_iter().map(|x| x.as_primitive()).collect();

        let response = ConfigResponse {
            releases,
            versions: ConfigVersions {
                fiat_on_ramp_assets: Self::version(fiat_on_ramp_assets),
                fiat_off_ramp_assets: Self::version(fiat_off_ramp_assets),
                swap_assets: FiatAssets::version(&swap_assets) as i32,
            },
            swap: SwapConfig {
                enabled_providers: SwapProvider::all().iter().map(|x| x.as_ref().to_string()).collect(),
            },
        };
        Ok(response)
    }

    fn version(assets: Vec<AssetBasic>) -> i32 {
        FiatAssets::version(&assets.into_iter().map(|x| x.asset.id.to_string()).collect::<Vec<String>>()) as i32
    }
}
