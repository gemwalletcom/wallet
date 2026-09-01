use primitives::{Wallet, WalletType};

pub fn shows_rewards(wallets: &[Wallet]) -> bool {
    wallets.is_empty() || wallets.iter().any(|wallet| wallet.wallet_type == WalletType::Multicoin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wallet(wallet_type: WalletType) -> Wallet {
        Wallet {
            wallet_type,
            ..Wallet::mock_with_accounts(vec![])
        }
    }

    #[test]
    fn test_rewards_need_a_multicoin_wallet_but_stay_visible_before_wallets_load() {
        assert!(shows_rewards(&[]));
        assert!(!shows_rewards(&[wallet(WalletType::Single)]));
        assert!(shows_rewards(&[wallet(WalletType::Single), wallet(WalletType::Multicoin)]));
    }
}
