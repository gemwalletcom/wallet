use crate::models::{TokenInfo, TokenInfoExtra};

impl TokenInfo {
    pub fn mock_with_marketplace(marketplace: Option<&str>) -> Self {
        Self {
            valid: true,
            name: Some("Collection".to_string()),
            description: None,
            image: None,
            extra: Some(TokenInfoExtra {
                domain: None,
                marketplace: marketplace.map(str::to_string),
            }),
        }
    }
}
