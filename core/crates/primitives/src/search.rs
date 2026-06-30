use crate::{AssetBasic, NFTCollection, PerpetualSearchData};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Sendable, Equatable, Hashable, Identifiable")]
#[serde(rename_all = "camelCase")]
pub struct AssetList {
    pub id: String,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Sendable")]
pub struct SearchResponse {
    pub assets: Vec<AssetBasic>,
    pub perpetuals: Vec<PerpetualSearchData>,
    pub nfts: Vec<NFTCollection>,
    pub lists: Vec<AssetList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Codable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum SearchItemType {
    Asset,
    Perpetual,
    Nft,
    List,
}
