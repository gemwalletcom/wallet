use primitives::{Account, Chain, Wallet};

pub const AUTH_CHAIN: Chain = Chain::Ethereum;

pub fn auth_account(wallet: &Wallet) -> Option<&Account> {
    wallet.account(AUTH_CHAIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wallet(chains: &[Chain]) -> Wallet {
        Wallet::mock_with_accounts(chains.iter().map(|chain| Account::mock(*chain, &format!("{chain}-address"))).collect())
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
