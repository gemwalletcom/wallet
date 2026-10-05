use gem_client::{ClientExt, ReqwestClient};
use primitives::{GEM_ANDROID_PACKAGE_ID, GEM_IOS_BUNDLE_ID, PlatformStore};
use std::error::Error;
use std::sync::Arc;

use super::repository::Repository;

use super::model::{FdroidPackageResponse, GitHubRepository, HuaweiStoreResponse, ITunesLookupResponse, SamsungStoreDetail, SolanaStoreRelease};
use super::store_target::{HuaweiAppRequest, StoreTarget};

pub struct VersionUpdater {
    repository: Arc<dyn Repository>,
    client: reqwest::Client,
}

impl VersionUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self {
            repository,
            client: gem_client::reqwest_client(),
        }
    }

    pub fn stores() -> &'static [PlatformStore] {
        &[
            PlatformStore::AppStore,
            PlatformStore::ApkUniversal,
            PlatformStore::Fdroid,
            PlatformStore::Huawei,
            PlatformStore::SamsungStore,
            PlatformStore::SolanaStore,
        ]
    }

    pub async fn update_store(&self, store: PlatformStore) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        if !self.repository.is_update_enabled(store).await? {
            return Ok(None);
        }

        let version = self.get_store_version(store).await?;
        self.repository.set_release_version(store, version.clone()).await?;

        Ok(Some(version))
    }

    async fn get_store_version(&self, store: PlatformStore) -> Result<String, Box<dyn Error + Send + Sync>> {
        match store {
            PlatformStore::AppStore => self.get_app_store_version().await,
            PlatformStore::ApkUniversal => self.get_github_version().await,
            PlatformStore::Fdroid => self.get_fdroid_version().await,
            PlatformStore::Huawei => self.get_huawei_version().await,
            PlatformStore::SamsungStore => self.get_samsung_version().await,
            PlatformStore::SolanaStore => self.get_solana_store_version().await,
            _ => Err(format!("unsupported store: {:?}", store).into()),
        }
    }

    fn store(&self, target: &StoreTarget) -> ReqwestClient {
        ReqwestClient::new(target.host().to_string(), self.client.clone())
    }

    async fn get_app_store_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let target = StoreTarget::AppStoreLookup { bundle_id: GEM_IOS_BUNDLE_ID.to_string() };
        let response: ITunesLookupResponse = self.store(&target).get(target).await?;
        response.results.first().map(|r| r.version.clone()).ok_or_else(|| "no results".into())
    }

    async fn get_github_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let target = StoreTarget::GithubReleases;
        let response: Vec<GitHubRepository> = self.store(&target).get(target).await?;
        response
            .into_iter()
            .find(|x| !x.draft && !x.prerelease && x.assets.iter().any(|a| a.name.contains("gem_wallet_universal_")))
            .map(|r| r.name)
            .ok_or_else(|| "no releases".into())
    }

    async fn get_fdroid_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let target = StoreTarget::FdroidPackage { package: GEM_ANDROID_PACKAGE_ID.to_string() };
        let response: FdroidPackageResponse = self.store(&target).get(target).await?;
        response.latest_version().ok_or_else(|| "f-droid version not found".into())
    }

    async fn get_huawei_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let target = StoreTarget::HuaweiApp;
        let request = HuaweiAppRequest {
            pkg_name: GEM_ANDROID_PACKAGE_ID.to_string(),
        };
        let response: HuaweiStoreResponse = self.store(&target).post(target, &request).await?;
        response.app_info.map(|app| app.version).ok_or_else(|| "huawei version not found".into())
    }

    async fn get_samsung_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let target = StoreTarget::SamsungDetail { package: GEM_ANDROID_PACKAGE_ID.to_string() };
        let response: SamsungStoreDetail = self.store(&target).get(target).await?;
        match response.details {
            Some(details) => Ok(details.version),
            None => Err(response.error_message.unwrap_or_else(|| "no version found".to_string()).into()),
        }
    }

    async fn get_solana_store_version(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let target = StoreTarget::SolanaRelease { package: GEM_ANDROID_PACKAGE_ID.to_string() };
        let response: SolanaStoreRelease = self.store(&target).get(target).await?;
        Ok(response.version_name)
    }
}
