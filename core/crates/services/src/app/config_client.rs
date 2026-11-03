use std::error::Error;
use std::sync::Arc;

use fiat::IpAddressProvider;
use primitives::{ConfigResponse, ConfigVersions, Features, PlatformStore, Release, SwapConfig, SwapProvider};

use super::client_country;
use super::repository::Repository;
use crate::assets::AssetCatalogClient;

#[derive(Clone)]
pub struct ConfigClient {
    repository: Arc<dyn Repository>,
    ip_address_provider: Arc<dyn IpAddressProvider>,
    asset_catalog: Arc<AssetCatalogClient>,
}

impl ConfigClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, ip_address_provider: Arc<dyn IpAddressProvider>, asset_catalog: Arc<AssetCatalogClient>) -> Self {
        Self {
            repository,
            ip_address_provider,
            asset_catalog,
        }
    }

    pub async fn get_config(&self, ip_address: &str, country: Option<&str>, store: Option<PlatformStore>) -> Result<ConfigResponse, Box<dyn Error + Send + Sync>> {
        let (features, releases, catalog) = tokio::try_join!(self.get_features(ip_address, country), self.get_releases(store), self.asset_catalog.get())?;

        let response = ConfigResponse {
            features,
            releases,
            versions: ConfigVersions {
                fiat_on_ramp_assets: catalog.fiat_on_ramp_assets.version as i32,
                fiat_off_ramp_assets: catalog.fiat_off_ramp_assets.version as i32,
                swap_assets: catalog.swap_assets.version as i32,
            },
            swap: SwapConfig {
                enabled_providers: SwapProvider::all().iter().map(|provider| provider.as_ref().to_string()).collect(),
            },
        };
        Ok(response)
    }

    async fn get_features(&self, ip_address: &str, country: Option<&str>) -> Result<Features, Box<dyn Error + Send + Sync>> {
        let country = client_country(self.ip_address_provider.as_ref(), ip_address, country).await?;
        let policies = self.repository.get_features().await?;
        Ok(Features::for_country(&policies, &country))
    }

    async fn get_releases(&self, store: Option<PlatformStore>) -> Result<Vec<Release>, Box<dyn Error + Send + Sync>> {
        Ok(releases_for_store(self.repository.get_releases().await?, store))
    }
}

fn releases_for_store(releases: Vec<Release>, store: Option<PlatformStore>) -> Vec<Release> {
    releases
        .into_iter()
        .filter(|release| match store {
            Some(store) => release.store == store,
            // Remove this exclusion when clients without store-scoped config are no longer supported.
            None => release.store != PlatformStore::MacDmg,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_releases_for_store() {
        let app_store = Release::new(PlatformStore::AppStore, "2.0.0".into(), true);
        let google_play = Release::new(PlatformStore::GooglePlay, "3.0.0".into(), false);
        let mac = Release::new(PlatformStore::MacDmg, "4.0.0".into(), false);
        let local = Release::new(PlatformStore::Local, "5.0.0".into(), false);
        let releases = vec![mac.clone(), app_store.clone(), google_play.clone(), local.clone()];

        assert_eq!(releases_for_store(releases.clone(), None), vec![app_store.clone(), google_play.clone(), local]);
        assert_eq!(releases_for_store(releases.clone(), Some(PlatformStore::MacDmg)), vec![mac]);
        assert_eq!(releases_for_store(releases.clone(), Some(PlatformStore::AppStore)), vec![app_store]);
        assert_eq!(releases_for_store(releases.clone(), Some(PlatformStore::GooglePlay)), vec![google_play]);
        assert_eq!(releases_for_store(releases, Some(PlatformStore::Fdroid)), vec![]);
        assert_eq!(releases_for_store(vec![], None), vec![]);
    }
}
