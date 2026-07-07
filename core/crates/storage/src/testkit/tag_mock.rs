use crate::models::AssetTagRow;
use primitives::AssetId;

impl AssetTagRow {
    pub fn mock_with_tag(asset_id: AssetId, tag_id: &str) -> Self {
        Self {
            asset_id: asset_id.into(),
            tag_id: tag_id.to_string(),
            order: None,
        }
    }
}
