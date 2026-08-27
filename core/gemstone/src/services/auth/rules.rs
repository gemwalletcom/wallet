use primitives::{Account, Chain, Wallet};

pub const AUTH_CHAIN: Chain = Chain::Ethereum;

pub fn auth_account(wallet: &Wallet) -> Option<&Account> {
    wallet.account(AUTH_CHAIN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{WalletId, WalletSource, WalletType};

    fn wallet(chains: &[Chain]) -> Wallet {
        Wallet {
            id: WalletId::Multicoin("0x1".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type: WalletType::Multicoin,
            accounts: chains
                .iter()
                .map(|chain| Account {
                    chain: *chain,
                    address: format!("{chain}-address"),
                    derivation_path: String::new(),
                    extended_public_key: None,
                })
                .collect(),
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        }
    }

    #[test]
    fn test_auth_account_uses_ethereum() {
        assert_eq!(
            auth_account(&wallet(&[Chain::Bitcoin, Chain::Ethereum])).map(|account| account.address.as_str()),
            Some("ethereum-address")
        );
        assert!(auth_account(&wallet(&[Chain::Bitcoin])).is_none());
    }
}
