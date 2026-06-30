use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ITunesLookupResponse {
    pub results: Vec<ITunesLoopUpResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ITunesLoopUpResult {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<GitHubRepositoryAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepositoryAsset {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamsungStoreDetail {
    #[serde(rename = "DetailMain")]
    pub details: Option<SamsungStoreDetails>,
    #[serde(rename = "errMsg")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamsungStoreDetails {
    #[serde(rename = "contentBinaryVersion")]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaStoreRelease {
    pub version_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HuaweiStoreResponse {
    pub app_info: Option<HuaweiStoreAppInfo>,
}

#[derive(Deserialize)]
pub struct HuaweiStoreAppInfo {
    pub version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FdroidPackageResponse {
    pub packages: Vec<FdroidPackage>,
}

impl FdroidPackageResponse {
    pub fn latest_version(&self) -> Option<String> {
        self.packages.first().map(|package| package.version_name.clone())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FdroidPackage {
    pub version_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdroid_latest_version() {
        let response: FdroidPackageResponse = serde_json::from_str(
            r#"{
                "packages": [
                    { "versionName": "2.0-alpha10", "versionCode": 2000010 },
                    { "versionName": "1.23.2", "versionCode": 1023052 }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(response.latest_version(), Some("2.0-alpha10".to_string()));
    }

    #[test]
    fn test_fdroid_latest_version_missing() {
        let response: FdroidPackageResponse = serde_json::from_str(r#"{ "packages": [] }"#).unwrap();

        assert_eq!(response.latest_version(), None);
    }
}
