use primitives::{WalletId, WalletSource, WalletType};

use crate::NewWallet;

impl NewWallet {
    pub fn mock(address: &str) -> Self {
        Self {
            wallet_id: WalletId::Multicoin(address.to_string()),
            wallet_type: WalletType::Multicoin,
            source: WalletSource::Import,
        }
    }
}
