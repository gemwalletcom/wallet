use primitives::Wallet;

use crate::wallet_connect::WalletConnectResponseType;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: String,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectResponse {
    Response { value: WalletConnectResponseType },
    Null,
    MethodNotFound,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSessionWallets {
    pub default_wallet: Wallet,
    pub wallets: Vec<Wallet>,
}
