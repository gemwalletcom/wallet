use std::collections::HashMap;

use primitives::AssetList;
use serde::{Deserialize, Serialize};

pub const ASSET_LISTS_INDEX_NAME: &str = "asset_lists";
pub const ASSET_LISTS_FILTERS: &[&str] = &[];
pub const ASSET_LISTS_SEARCH_ATTRIBUTES: &[&str] = &["name", "id"];
pub const ASSET_LISTS_RANKING_RULES: &[&str] = &["words", "typo", "proximity", "attribute", "exactness"];
pub const ASSET_LISTS_SORTS: &[&str] = &[];

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetListDocument {
    pub id: String,
    pub name: String,
    pub chain_counts: HashMap<String, u32>,
}

impl AssetListDocument {
    pub fn new(id: String, name: String, chain_counts: HashMap<String, u32>) -> Self {
        Self { id, name, chain_counts }
    }

    pub fn as_primitive(&self, chains: &[String]) -> Option<AssetList> {
        let count = self.chain_counts.iter().filter(|(chain, _)| chains.is_empty() || chains.contains(chain)).map(|(_, count)| count).sum();
        (count > 0).then(|| AssetList {
            id: self.id.clone(),
            name: self.name.clone(),
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_primitive() {
        let document = AssetListDocument::mock();

        assert_eq!(document.as_primitive(&[]), Some(AssetList::mock(6)));
        assert_eq!(document.as_primitive(&["ethereum".to_string()]), Some(AssetList::mock(4)));
        assert_eq!(document.as_primitive(&["bitcoin".to_string(), "solana".to_string()]), Some(AssetList::mock(2)));
        assert_eq!(document.as_primitive(&["bitcoin".to_string()]), None);
    }
}
