use primitives::{AssetFiatValue, PerpetualBalance, Wallet, WalletType};

use crate::services::assets::model::{GemHeaderActions, GemHeaderButton, GemHeaderButtonKind};

pub fn shows_initial_loading(initial_load_completed: bool, assets_timestamp: u64) -> bool {
    !initial_load_completed && assets_timestamp == 0
}

pub fn wallet_balances(balances: Vec<AssetFiatValue>, perpetual: Option<PerpetualBalance>) -> Vec<AssetFiatValue> {
    balances
        .into_iter()
        .chain(perpetual.map(|balance| AssetFiatValue {
            amount: balance.available + balance.reserved,
            price: 1.0,
            price_change_percentage_24h: 0.0,
        }))
        .collect()
}

pub fn header_actions(wallet: &Wallet, is_enabled: bool) -> GemHeaderActions {
    match wallet.wallet_type {
        WalletType::View => GemHeaderActions::WatchOnly,
        WalletType::Multicoin | WalletType::Single | WalletType::PrivateKey => GemHeaderActions::Buttons {
            buttons: header_buttons(wallet, is_enabled),
        },
    }
}

fn header_buttons(wallet: &Wallet, is_enabled: bool) -> Vec<GemHeaderButton> {
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

    fn buttons(wallet: &Wallet, is_enabled: bool) -> Vec<GemHeaderButton> {
        match header_actions(wallet, is_enabled) {
            GemHeaderActions::Buttons { buttons } => buttons,
            GemHeaderActions::WatchOnly => panic!("the header offers no buttons"),
        }
    }

    fn kinds(wallet: &Wallet) -> Vec<GemHeaderButtonKind> {
        buttons(wallet, true).into_iter().map(|button| button.kind).collect()
    }

    #[test]
    fn test_wallet_balances_count_perpetual_collateral_at_par_without_a_day_change() {
        let eth = AssetFiatValue {
            amount: 2.0,
            price: 10.0,
            price_change_percentage_24h: 5.0,
        };
        let collateral = PerpetualBalance {
            available: 30.0,
            reserved: 20.0,
            withdrawable: 25.0,
        };
        assert_eq!(wallet_balances(vec![eth.clone()], None), vec![eth.clone()]);
        assert_eq!(
            wallet_balances(vec![eth.clone()], Some(collateral)),
            vec![
                eth,
                AssetFiatValue {
                    amount: 50.0,
                    price: 1.0,
                    price_change_percentage_24h: 0.0
                }
            ],
            "collateral is what is available plus what positions hold, not what can be withdrawn"
        );
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
        assert_eq!(header_actions(&wallet(WalletType::View, Chain::Ethereum), true), GemHeaderActions::WatchOnly);
        assert!(buttons(&wallet(WalletType::Multicoin, Chain::Ethereum), false).iter().all(|button| !button.is_enabled));
    }
}
