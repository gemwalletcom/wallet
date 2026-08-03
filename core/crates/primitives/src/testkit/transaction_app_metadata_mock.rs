use crate::TransactionAppMetadata;

impl TransactionAppMetadata {
    pub fn mock() -> Self {
        TransactionAppMetadata {
            name: "Test Dapp".to_string(),
            description: None,
            url: Some("https://example.com".to_string()),
            icon: Some("https://example.com/icon.png".to_string()),
        }
    }
}
