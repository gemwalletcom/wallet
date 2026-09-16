use crate::{AddressChains, Chain, Device, DeviceSubscription, WalletId, WalletSource, WalletSubscription, WalletSubscriptionChains};

impl DeviceSubscription {
    pub fn mock() -> Self {
        Self {
            wallet_row_id: 1,
            device: Device::mock(),
            wallet_id: WalletId::Multicoin("0xABC".to_string()),
            chain: Chain::Ethereum,
            address: "0xABC".to_string(),
        }
    }
}

impl WalletSubscription {
    pub fn mock(wallet_id: &str, subscriptions: Vec<AddressChains>) -> Self {
        Self {
            wallet_id: WalletId::Multicoin(wallet_id.to_string()),
            source: Some(WalletSource::Import),
            subscriptions,
        }
    }
}

impl WalletSubscriptionChains {
    pub fn mock(wallet_id: &str, chains: Vec<Chain>) -> Self {
        Self {
            wallet_id: WalletId::Multicoin(wallet_id.to_string()),
            chains,
        }
    }
}
