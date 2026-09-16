use primitives::{Account, Chain, Wallet};

pub const AUTH_CHAIN: Chain = Chain::Ethereum;

pub fn auth_account(wallet: &Wallet) -> Option<&Account> {
    wallet.account(AUTH_CHAIN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_account_uses_ethereum() {
        assert_eq!(
            auth_account(&Wallet::mock_with_accounts(vec![
                Account::mock(Chain::Bitcoin, "bitcoin-address"),
                Account::mock(Chain::Ethereum, "ethereum-address")
            ]))
            .map(|account| account.address.as_str()),
            Some("ethereum-address")
        );
        assert!(auth_account(&Wallet::mock_with_chains(&[Chain::Bitcoin])).is_none());
    }
}
