use crate::WalletConnectAppMetadata;

impl WalletConnectAppMetadata {
    pub fn mock() -> Self {
        WalletConnectAppMetadata {
            name: "Test Dapp".to_string(),
            description: "Test Dapp".to_string(),
            url: "https://example.com".to_string(),
            icon: "https://example.com/icon.png".to_string(),
        }
    }
}
