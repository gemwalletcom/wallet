use primitives::{Wallet, WalletType};

pub fn shows_rewards(wallets: &[Wallet]) -> bool {
    wallets.is_empty() || wallets.iter().any(|wallet| wallet.wallet_type == WalletType::Multicoin)
}

pub fn rewards_wallets(wallets: Vec<Wallet>) -> Vec<Wallet> {
    wallets.into_iter().filter(|wallet| wallet.wallet_type == WalletType::Multicoin).collect()
}

pub fn can_choose_wallet(wallets: &[Wallet]) -> bool {
    wallets.len() > 1
}

pub fn rewards_wallet(current: Option<Wallet>, wallets: &[Wallet]) -> Option<Wallet> {
    current.filter(|wallet| wallet.wallet_type == WalletType::Multicoin).or_else(|| wallets.first().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewards_wallet_prefers_the_current_multicoin_wallet() {
        let current = Wallet {
            id: primitives::WalletId::Multicoin("current".to_string()),
            ..Wallet::mock_with_type(WalletType::Multicoin, &[])
        };
        let other = Wallet {
            id: primitives::WalletId::Multicoin("other".to_string()),
            ..Wallet::mock_with_type(WalletType::Multicoin, &[])
        };
        let wallets = rewards_wallets(vec![Wallet::mock_with_type(WalletType::Single, &[]), other.clone(), current.clone()]);

        assert_eq!(wallets.iter().map(|wallet| wallet.id.clone()).collect::<Vec<_>>(), vec![other.id.clone(), current.id.clone()]);
        assert_eq!(rewards_wallet(Some(current.clone()), &wallets).map(|wallet| wallet.id), Some(current.id));
        assert_eq!(rewards_wallet(Some(Wallet::mock_with_type(WalletType::Single, &[])), &wallets).map(|wallet| wallet.id), Some(other.id));
        assert!(rewards_wallet(None, &[]).is_none());
    }

    #[test]
    fn test_a_wallet_can_be_chosen_only_when_there_is_more_than_one() {
        let wallet = Wallet::mock_with_type(WalletType::Multicoin, &[]);

        assert!(!can_choose_wallet(&[]));
        assert!(!can_choose_wallet(std::slice::from_ref(&wallet)));
        assert!(can_choose_wallet(&[wallet.clone(), wallet]));
    }

    #[test]
    fn test_rewards_need_a_multicoin_wallet_but_stay_visible_before_wallets_load() {
        assert!(shows_rewards(&[]));
        assert!(!shows_rewards(&[Wallet::mock_with_type(WalletType::Single, &[])]));
        assert!(shows_rewards(&[Wallet::mock_with_type(WalletType::Single, &[]), Wallet::mock_with_type(WalletType::Multicoin, &[])]));
    }
}
