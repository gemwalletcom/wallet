use primitives::AssetList;
use serde::{Deserialize, Serialize};

pub const ASSET_LISTS_INDEX_NAME: &str = "asset_lists";
pub const ASSET_LISTS_FILTERS: &[&str] = &[];
pub const ASSET_LISTS_SEARCH_ATTRIBUTES: &[&str] = &["name", "id"];
pub const ASSET_LISTS_RANKING_RULES: &[&str] = &["words", "typo", "proximity", "attribute", "exactness"];
pub const ASSET_LISTS_SORTS: &[&str] = &[];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetListDocument {
    pub id: String,
    pub name: String,
    pub count: u32,
}

impl AssetListDocument {
    pub fn new(list: AssetList) -> Self {
        Self {
            id: list.id,
            name: list.name,
            count: list.count,
        }
    }
}

impl From<AssetList> for AssetListDocument {
    fn from(list: AssetList) -> Self {
        Self::new(list)
    }
}

impl From<AssetListDocument> for AssetList {
    fn from(document: AssetListDocument) -> Self {
        Self {
            id: document.id,
            name: document.name,
            count: document.count,
        }
    }
}
