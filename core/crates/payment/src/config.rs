use crate::wallet_connect_pay::WalletConnectPayAuth;

#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub wallet_connect_pay: WalletConnectPayAuth,
}

impl PaymentConfig {
    pub fn new(wallet_connect_pay: WalletConnectPayAuth) -> Self {
        Self { wallet_connect_pay }
    }
}
