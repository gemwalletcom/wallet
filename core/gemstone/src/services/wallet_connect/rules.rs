use std::collections::HashSet;

use primitives::{Chain, Wallet, WalletId, WalletType};

use crate::wallet_connect::WalletConnect;

pub fn session_wallets(wallets: Vec<Wallet>, required: &[Chain], optional: &[Chain]) -> Vec<Wallet> {
    let wallet_connect = WalletConnect::new();
    let mut supported: Vec<Wallet> = wallets
        .into_iter()
        .filter(|wallet| wallet.wallet_type != WalletType::View && supports(wallet, required, optional, &wallet_connect))
        .collect();
    supported.sort_by_key(|wallet| wallet_type_rank(&wallet.wallet_type));
    supported
}

pub fn default_wallet(wallets: &[Wallet], current_wallet_id: Option<WalletId>) -> Option<Wallet> {
    wallets
        .iter()
        .find(|wallet| Some(&wallet.id) == current_wallet_id.as_ref())
        .or_else(|| wallets.first())
        .cloned()
}

pub fn session_chains(wallet: &Wallet, supported: &[Chain]) -> Vec<Chain> {
    let wallet_chains: HashSet<Chain> = wallet.accounts.iter().map(|account| account.chain).collect();
    supported.iter().copied().filter(|chain| wallet_chains.contains(chain)).collect()
}

fn supports(wallet: &Wallet, required: &[Chain], optional: &[Chain], wallet_connect: &WalletConnect) -> bool {
    let chains: HashSet<Chain> = wallet
        .accounts
        .iter()
        .map(|account| account.chain)
        .filter(|chain| wallet_connect.get_namespace(chain.as_ref().to_string()).is_some())
        .collect();
    if chains.is_empty() {
        return false;
    }
    if !required.is_empty() {
        return required.iter().all(|chain| chains.contains(chain));
    }
    optional.is_empty() || optional.iter().any(|chain| chains.contains(chain))
}

fn wallet_type_rank(wallet_type: &WalletType) -> u8 {
    match wallet_type {
        WalletType::Multicoin => 0,
        WalletType::Single => 1,
        WalletType::PrivateKey => 2,
        WalletType::View => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, WalletSource};

    fn wallet(id: &str, wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            id: WalletId::Multicoin(id.to_string()),
            external_id: None,
            name: id.to_string(),
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
    fn test_session_wallets() {
        let multicoin = wallet("multi", WalletType::Multicoin, &[Chain::Ethereum, Chain::Solana]);
        let single = wallet("single", WalletType::Single, &[Chain::Ethereum]);
        let view = wallet("view", WalletType::View, &[Chain::Ethereum]);
        let bitcoin_only = wallet("btc", WalletType::Single, &[Chain::Bitcoin]);
        let wallets = vec![single.clone(), view, bitcoin_only, multicoin.clone()];

        let required = session_wallets(wallets.clone(), &[Chain::Ethereum, Chain::Solana], &[]);
        assert_eq!(required.iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["multi"]);

        let optional = session_wallets(wallets.clone(), &[], &[Chain::Ethereum]);
        assert_eq!(optional.iter().map(|wallet| wallet.name.as_str()).collect::<Vec<_>>(), vec!["multi", "single"]);

        let any = session_wallets(wallets, &[], &[]);
        assert_eq!(any.len(), 2);
    }

    #[test]
    fn test_default_wallet_prefers_current() {
        let first = wallet("first", WalletType::Multicoin, &[Chain::Ethereum]);
        let second = wallet("second", WalletType::Multicoin, &[Chain::Ethereum]);
        let wallets = vec![first.clone(), second.clone()];
        assert_eq!(default_wallet(&wallets, Some(second.id.clone())).map(|wallet| wallet.name), Some("second".to_string()));
        assert_eq!(
            default_wallet(&wallets, Some(WalletId::Multicoin("other".to_string()))).map(|wallet| wallet.name),
            Some("first".to_string())
        );
        assert!(default_wallet(&[], None).is_none());
    }

    #[test]
    fn test_session_chains_keeps_supported_order() {
        let wallet = wallet("w", WalletType::Multicoin, &[Chain::Solana, Chain::Ethereum, Chain::Bitcoin]);
        assert_eq!(
            session_chains(&wallet, &[Chain::Ethereum, Chain::Solana, Chain::Tron]),
            vec![Chain::Ethereum, Chain::Solana]
        );
    }
}
