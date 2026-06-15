use primitives::{Asset, Perpetual, PerpetualSearchData};
use serde::{Deserialize, Serialize};

pub const PERPETUALS_INDEX_NAME: &str = "perpetuals";
pub const PERPETUALS_FILTERS: &[&str] = &[
    "perpetual.name",
    "perpetual.identifier",
    "perpetual.provider",
    "perpetual.price",
    "perpetual.volume24h",
    "tags",
];
pub const PERPETUALS_SEARCH_ATTRIBUTES: &[&str] = &["perpetual.name", "perpetual.identifier", "perpetual.provider"];
pub const PERPETUALS_RANKING_RULES: &[&str] = &["words", "typo", "perpetual.volume24h:desc", "proximity", "attribute", "exactness"];

pub const PERPETUALS_SORTS: &[&str] = &["perpetual.volume24h"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerpetualDocument {
    pub id: String,
    pub perpetual: Perpetual,
    pub asset: Asset,
    pub tags: Option<Vec<String>>,
}

impl From<PerpetualDocument> for PerpetualSearchData {
    fn from(doc: PerpetualDocument) -> Self {
        Self {
            perpetual: doc.perpetual,
            asset: doc.asset,
        }
    }
}
