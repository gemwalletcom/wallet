use crate::{ApplicationMetadata, ApplicationMetadataSource};

impl ApplicationMetadata {
    pub fn mock() -> Self {
        ApplicationMetadata {
            name: "Test Dapp".to_string(),
            description: "Test Dapp".to_string(),
            url: "https://example.com".to_string(),
            icon: "https://example.com/icon.png".to_string(),
            source: ApplicationMetadataSource::WalletConnect,
        }
    }
}
