use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct StonfiAssetsQuery {
    pub condition: &'static str,
    pub sort_by: [&'static str; 1],
    pub limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct StonfiAssetsResponse {
    pub asset_list: Vec<StonfiAsset>,
}

#[derive(Debug, Deserialize)]
pub struct StonfiAsset {
    pub contract_address: String,
    pub kind: StonfiAssetKind,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub enum StonfiAssetKind {
    Ton,
    Jetton,
    #[serde(other)]
    Unsupported,
}
