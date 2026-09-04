use gem_client::{Target, build_path_with_query};
use serde::Serialize;

#[derive(Clone, Debug)]
pub enum StoreTarget {
    AppStoreLookup { bundle_id: String },
    GithubReleases,
    FdroidPackage { package: String },
    HuaweiApp,
    SamsungDetail { package: String },
    SolanaRelease { package: String },
}

impl StoreTarget {
    pub fn host(&self) -> &'static str {
        match self {
            Self::AppStoreLookup { .. } => "https://itunes.apple.com",
            Self::GithubReleases => "https://api.github.com",
            Self::FdroidPackage { .. } => "https://f-droid.org",
            Self::HuaweiApp => "https://web-dre.hispace.dbankcloud.com",
            Self::SamsungDetail { .. } => "https://galaxystore.samsung.com",
            Self::SolanaRelease { .. } => "https://publish.solanamobile.com",
        }
    }
}

impl Target for StoreTarget {
    fn path(&self) -> String {
        match self {
            Self::AppStoreLookup { bundle_id } => build_path_with_query("/lookup", &[("bundleId", bundle_id)]),
            Self::GithubReleases => "/repos/gemwalletcom/wallet/releases".to_string(),
            Self::FdroidPackage { package } => format!("/api/v1/packages/{package}"),
            Self::HuaweiApp => "/edge/single/filtered".to_string(),
            Self::SamsungDetail { package } => format!("/api/detail/{package}"),
            Self::SolanaRelease { package } => format!("/api/{package}/release"),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HuaweiAppRequest {
    pub pkg_name: String,
}
