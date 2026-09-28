use primitives::TagVisibility;

use crate::repositories::tag_repository::Tag;

impl Tag {
    pub fn mock(id: &str, visibility: TagVisibility) -> Self {
        Self {
            id: id.to_string(),
            name: id.to_string(),
            visibility,
            list_id: None,
        }
    }
}
