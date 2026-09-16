use crate::{Account, Chain, Wallet, WalletId, WalletSource, WalletType};

impl Wallet {
    pub fn mock() -> Self {
        Self::mock_with_accounts(vec![Account::mock(Chain::Ethereum, "address")])
    }

    pub fn mock_with_chains(chains: &[Chain]) -> Self {
        Self::mock_with_accounts(Account::mock_chains(chains, "address"))
    }

    pub fn mock_with_type(wallet_type: WalletType, chains: &[Chain]) -> Self {
        Self {
            wallet_type,
            ..Self::mock_with_chains(chains)
        }
    }

    pub fn mock_with_id(id: WalletId, chains: &[Chain]) -> Self {
        Self {
            wallet_type: id.wallet_type(),
            id,
            ..Self::mock_with_chains(chains)
        }
    }

    pub fn mock_with_accounts(accounts: Vec<Account>) -> Self {
        Self {
            id: WalletId::Multicoin("0x1".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type: WalletType::Multicoin,
            accounts,
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        }
    }
}
