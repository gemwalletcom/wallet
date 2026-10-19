use crate::models::{RunGetMethodResult, StackEntry, TokenInfo, TokenInfoExtra};

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

impl RunGetMethodResult {
    pub fn mock(exit_code: i32, stack: Vec<StackEntry>) -> Self {
        Self { stack, exit_code }
    }
}
