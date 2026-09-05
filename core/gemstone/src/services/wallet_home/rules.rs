use primitives::{Wallet, WalletType};

use crate::services::assets::model::{GemHeaderButton, GemHeaderButtonKind};

pub fn shows_initial_loading(initial_load_completed: bool, assets_timestamp: u64) -> bool {
    !initial_load_completed && assets_timestamp == 0
}

pub fn header_buttons(wallet: &Wallet, is_enabled: bool) -> Vec<GemHeaderButton> {
    [
        Some(GemHeaderButtonKind::Send),
        Some(GemHeaderButtonKind::Receive),
        Some(GemHeaderButtonKind::Buy),
        swaps(wallet).then_some(GemHeaderButtonKind::Swap),
    ]
    .into_iter()
    .flatten()
    .map(|kind| GemHeaderButton { kind, is_enabled })
    .collect()
}

fn swaps(wallet: &Wallet) -> bool {
    match wallet.wallet_type {
        WalletType::Multicoin => true,
        WalletType::Single | WalletType::PrivateKey => wallet.accounts.first().is_some_and(|account| account.chain.is_swap_supported()),
        WalletType::View => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, Chain};

    fn wallet(wallet_type: WalletType, chain: Chain) -> Wallet {
        Wallet {
            wallet_type,
            ..Wallet::mock_with_accounts(vec![Account {
                chain,
                address: "address".to_string(),
                derivation_path: String::new(),
                extended_public_key: None,
            }])
        }
    }

    fn kinds(wallet: &Wallet) -> Vec<GemHeaderButtonKind> {
        header_buttons(wallet, true).into_iter().map(|button| button.kind).collect()
    }

    #[test]
    fn test_shows_initial_loading_only_before_the_first_discovery() {
        assert!(shows_initial_loading(false, 0));
        assert!(!shows_initial_loading(true, 0));
        assert!(!shows_initial_loading(false, 1));
        assert!(!shows_initial_loading(true, 1));
    }

    #[test]
    fn test_the_header_offers_swap_to_multicoin_and_swappable_single_chain_wallets_only() {
        use GemHeaderButtonKind::*;
        assert_eq!(kinds(&wallet(WalletType::Multicoin, Chain::Bitcoin)), vec![Send, Receive, Buy, Swap]);
        assert_eq!(kinds(&wallet(WalletType::Single, Chain::Ethereum)), vec![Send, Receive, Buy, Swap]);
        assert_eq!(kinds(&wallet(WalletType::PrivateKey, Chain::Solana)), vec![Send, Receive, Buy, Swap]);
        assert_eq!(kinds(&wallet(WalletType::Single, Chain::Mayachain)), vec![Send, Receive, Buy]);
        assert_eq!(kinds(&wallet(WalletType::View, Chain::Ethereum)), vec![Send, Receive, Buy]);
        assert!(
            header_buttons(&wallet(WalletType::Multicoin, Chain::Ethereum), false)
                .iter()
                .all(|button| !button.is_enabled)
        );
    }
}
