use primitives::{Wallet, WalletId, WalletType};

pub fn shows_rewards(wallets: &[Wallet]) -> bool {
    wallets.is_empty() || wallets.iter().any(|wallet| wallet.wallet_type == WalletType::Multicoin)
}

pub fn rewards_wallets(wallets: Vec<Wallet>) -> Vec<Wallet> {
    wallets.into_iter().filter(|wallet| wallet.wallet_type == WalletType::Multicoin).collect()
}

pub fn can_choose_wallet(wallets: &[Wallet]) -> bool {
    wallets.len() > 1
}

pub fn rewards_wallet<'a>(wallets: &'a [Wallet], requested: Option<&WalletId>, current: Option<&WalletId>) -> Option<&'a Wallet> {
    let find = |wallet_id: Option<&WalletId>| wallet_id.and_then(|wallet_id| wallets.iter().find(|wallet| &wallet.id == wallet_id));
    find(requested).or_else(|| find(current)).or_else(|| wallets.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn multicoin(id: &str) -> Wallet {
        Wallet {
            id: WalletId::Multicoin(id.to_string()),
            ..Wallet::mock_with_type(WalletType::Multicoin, &[])
        }
    }

    #[test]
    fn test_rewards_wallet_keeps_the_chosen_wallet_then_the_current_one_then_the_first() {
        let single = Wallet::mock_with_type(WalletType::Single, &[]);
        let wallets = rewards_wallets(vec![single.clone(), multicoin("first"), multicoin("current"), multicoin("chosen")]);
        let chosen = WalletId::Multicoin("chosen".to_string());
        let current = WalletId::Multicoin("current".to_string());
        let first = WalletId::Multicoin("first".to_string());
        let selected = |requested: Option<&WalletId>, current: Option<&WalletId>| rewards_wallet(&wallets, requested, current).map(|wallet| wallet.id.clone());

        assert_eq!(wallets.iter().map(|wallet| wallet.id.clone()).collect::<Vec<_>>(), vec![first.clone(), current.clone(), chosen.clone()]);
        assert_eq!(selected(Some(&chosen), Some(&current)), Some(chosen.clone()));
        assert_eq!(selected(None, Some(&current)), Some(current.clone()));
        assert_eq!(
            selected(Some(&WalletId::Multicoin("deleted".to_string())), Some(&current)),
            Some(current.clone()),
            "a chosen wallet that is gone falls back to the current one"
        );
        assert_eq!(selected(None, Some(&single.id)), Some(first), "a current wallet that cannot earn rewards falls back to the first that can");
        assert!(rewards_wallet(&[], None, Some(&current)).is_none());
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
