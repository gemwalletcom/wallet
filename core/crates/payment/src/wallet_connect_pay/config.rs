#[derive(Debug, Clone)]
pub struct WalletConnectPayAuth {
    pub app_id: String,
    pub client_id: String,
}

impl WalletConnectPayAuth {
    pub fn new(app_id: String, client_id: String) -> Self {
        Self { app_id, client_id }
    }
}
