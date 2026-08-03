use crate::wallet_connect_pay::client::WalletConnectPayAuth;

pub const TEST_ACCOUNT_ETHEREUM: &str = "eip155:1:0x1085c5f70F7F7591D97da281A64688385455c2bD";
pub const TEST_ACCOUNT_POLYGON: &str = "eip155:137:0x1085c5f70F7F7591D97da281A64688385455c2bD";
pub const TEST_ACCOUNT_SOLANA: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:HA4hQMs22nCuRN7iLDBsBkboz2SnLM1WkNtzLo6xEDY5";
pub const TEST_APP_ID: &str = "app_1";
pub const TEST_CLIENT_ID: &str = "client_1";

impl WalletConnectPayAuth {
    pub fn mock() -> Self {
        Self {
            app_id: TEST_APP_ID.to_string(),
            client_id: TEST_CLIENT_ID.to_string(),
        }
    }
}
