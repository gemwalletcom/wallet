use primitives::{Account, Chain, Wallet, WalletId, WalletSource, WalletType};

use crate::keystore::GemKeystoreAccount;

pub fn view_wallet(name: String, chain: Chain, address: String) -> Wallet {
    Wallet {
        id: WalletId::View(chain, address.clone()),
        external_id: None,
        name,
        index: 0,
        wallet_type: WalletType::View,
        accounts: vec![Account {
            chain,
            address,
            derivation_path: String::new(),
            extended_public_key: Some(String::new()),
        }],
        is_pinned: false,
        image_url: None,
        source: WalletSource::Import,
    }
}

pub fn account(account: GemKeystoreAccount) -> Account {
    Account {
        chain: account.chain,
        address: account.address,
        derivation_path: account.derivation_path,
        extended_public_key: Some(account.public_key.unwrap_or_default()),
    }
}

pub fn next_wallet_index(wallets: &[Wallet]) -> i32 {
    wallets.iter().map(|wallet| wallet.index).max().map(|index| index + 1).unwrap_or(1)
}

pub fn missing_chains(wallet: &Wallet, chains: &[Chain]) -> Vec<Chain> {
    chains
        .iter()
        .copied()
        .filter(|chain| !wallet.accounts.iter().any(|account| account.chain == *chain))
        .collect()
}

pub fn wallets_missing_chains(wallets: Vec<Wallet>, chains: &[Chain]) -> Vec<(Wallet, Vec<Chain>)> {
    wallets
        .into_iter()
        .filter(|wallet| wallet.wallet_type == WalletType::Multicoin)
        .filter_map(|wallet| {
            let missing = missing_chains(&wallet, chains);
            (!missing.is_empty()).then_some((wallet, missing))
        })
        .collect()
}

pub fn next_current_wallet(wallets: &[Wallet]) -> Option<WalletId> {
    wallets
        .iter()
        .min_by_key(|wallet| (wallet_type_rank(&wallet.wallet_type), wallet.index))
        .map(|wallet| wallet.id.clone())
}

fn wallet_type_rank(wallet_type: &WalletType) -> u8 {
    match wallet_type {
        WalletType::Multicoin => 0,
        WalletType::Single => 1,
        WalletType::PrivateKey => 2,
        WalletType::View => 3,
    }
}

pub fn existing_wallet(wallets: &[Wallet], wallet_id: &WalletId, wallet_type: WalletType) -> Option<Wallet> {
    wallets.iter().find(|wallet| wallet.id == *wallet_id && wallet.wallet_type == wallet_type).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wallet(id: WalletId, wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            id,
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type,
            accounts: chains
                .iter()
                .map(|chain| Account {
                    chain: *chain,
                    address: "address".to_string(),
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
    fn test_wallets_missing_chains() {
        let multicoin = wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        let complete = wallet(WalletId::Multicoin("0x2".to_string()), WalletType::Multicoin, &[Chain::Ethereum, Chain::Bitcoin]);
        let single = wallet(WalletId::Single(Chain::Ethereum, "0x3".to_string()), WalletType::Single, &[Chain::Ethereum]);

        let result = wallets_missing_chains(vec![multicoin.clone(), complete, single], &[Chain::Ethereum, Chain::Bitcoin]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0.id, multicoin.id);
        assert_eq!(result[0].1, vec![Chain::Bitcoin]);
    }

    #[test]
    fn test_existing_wallet_requires_same_type() {
        let id = WalletId::Single(Chain::Bitcoin, "bc1".to_string());
        let wallets = vec![wallet(id.clone(), WalletType::Single, &[Chain::Bitcoin])];

        assert!(existing_wallet(&wallets, &id, WalletType::Single).is_some());
        assert!(existing_wallet(&wallets, &id, WalletType::PrivateKey).is_none());
        assert!(existing_wallet(&wallets, &WalletId::Multicoin("0x".to_string()), WalletType::Multicoin).is_none());
    }

    #[test]
    fn test_next_current_wallet_prefers_multicoin_then_lowest_index() {
        let mut view = wallet(WalletId::View(Chain::Ethereum, "0xv".to_string()), WalletType::View, &[Chain::Ethereum]);
        view.index = 0;
        let mut second = wallet(WalletId::Multicoin("0x2".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        second.index = 2;
        let mut first = wallet(WalletId::Multicoin("0x1".to_string()), WalletType::Multicoin, &[Chain::Ethereum]);
        first.index = 1;

        assert_eq!(next_current_wallet(&[view, second, first.clone()]), Some(first.id));
        assert_eq!(next_current_wallet(&[]), None);
    }

    #[test]
    fn test_view_wallet() {
        let result = view_wallet("Watch".to_string(), Chain::Ethereum, "0xabc".to_string());
        assert_eq!(result.id, WalletId::View(Chain::Ethereum, "0xabc".to_string()));
        assert_eq!(result.wallet_type, WalletType::View);
        assert_eq!(result.accounts.len(), 1);
        assert_eq!(result.accounts[0].address, "0xabc");
    }

    #[test]
    fn test_next_wallet_index_uses_highest_index() {
        assert_eq!(next_wallet_index(&[]), 1);
        let mut first = wallet(WalletId::Multicoin("1".into()), WalletType::Multicoin, &[]);
        first.index = 3;
        let mut second = wallet(WalletId::Multicoin("2".into()), WalletType::Single, &[]);
        second.index = 1;
        assert_eq!(next_wallet_index(&[first, second]), 4);
    }
}
