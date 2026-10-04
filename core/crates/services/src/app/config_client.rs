use std::error::Error;
use std::sync::Arc;

use fiat::IpAddressProvider;
use primitives::{AssetBasic, ConfigResponse, ConfigVersions, Features, FiatAssets, SwapConfig, SwapProvider};
use storage::{AssetFilter, AssetsRepository, Database, DatabaseError, FeaturesRepository, ReleasesRepository};

#[derive(Clone)]
pub struct ConfigClient {
    database: Database,
    ip_address_provider: Arc<dyn IpAddressProvider>,
}

impl ConfigClient {
    pub fn new(database: Database, ip_address_provider: Arc<dyn IpAddressProvider>) -> Self {
        Self { database, ip_address_provider }
    }

    pub async fn get_config(&self, ip_address: &str) -> Result<ConfigResponse, Box<dyn Error + Send + Sync>> {
        let features = self.get_features(ip_address).await?;
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
            features,
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

    async fn get_features(&self, ip_address: &str) -> Result<Features, Box<dyn Error + Send + Sync>> {
        let country = self.ip_address_provider.get_ip_address(ip_address).await?;
        let policies = self.database.run(FeaturesRepository::get_features).await?;
        Ok(Features::for_country(&policies, &country.alpha2))
    }

    fn version(assets: Vec<AssetBasic>) -> i32 {
        FiatAssets::version(&assets.into_iter().map(|asset| asset.asset.id.to_string()).collect::<Vec<String>>()) as i32
    }
}
