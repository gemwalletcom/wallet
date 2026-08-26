use primitives::{Account, Chain};

pub fn hyperliquid_account(accounts: &[Account]) -> Option<&Account> {
    accounts
        .iter()
        .find(|account| matches!(account.chain, Chain::Arbitrum | Chain::HyperCore | Chain::Hyperliquid))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(chain: Chain, address: &str) -> Account {
        Account {
            chain,
            address: address.into(),
            derivation_path: String::new(),
            extended_public_key: None,
        }
    }

    #[test]
    fn test_hyperliquid_account() {
        let accounts = [account(Chain::Bitcoin, "bc1"), account(Chain::HyperCore, "0xhl")];
        assert_eq!(hyperliquid_account(&accounts).map(|account| account.address.as_str()), Some("0xhl"));
        assert!(hyperliquid_account(&[account(Chain::Bitcoin, "bc1")]).is_none());
    }
}
