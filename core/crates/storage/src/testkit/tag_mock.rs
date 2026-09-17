use crate::models::{AssetTagRow, PerpetualTagRow, TagRow};
use crate::sql_types::TagVisibility;
use primitives::{AssetId, PerpetualId};

impl TagRow {
    pub fn mock(id: &str, visibility: TagVisibility) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            visibility,
            list_id: None,
        }
    }
}

impl AssetTagRow {
    pub fn mock_with_tag(asset_id: AssetId, tag_id: &str) -> Self {
        Self {
            asset_id: asset_id.into(),
            tag_id: tag_id.to_string(),
            order: None,
        }
    }
}

impl PerpetualTagRow {
    pub fn mock_with_tag(perpetual_id: PerpetualId, tag_id: &str) -> Self {
        Self {
            perpetual_id: perpetual_id.into(),
            tag_id: tag_id.to_string(),
            order: None,
        }
    }
}
