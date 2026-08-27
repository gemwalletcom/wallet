use std::collections::HashSet;

use primitives::{Account, AssetId, Chain};

pub fn hyperliquid_account(accounts: &[Account]) -> Option<&Account> {
    accounts
        .iter()
        .find(|account| matches!(account.chain, Chain::Arbitrum | Chain::HyperCore | Chain::Hyperliquid))
}

pub fn new_asset_ids(subscribed: &HashSet<AssetId>, asset_ids: Vec<AssetId>) -> Vec<AssetId> {
    let mut seen = subscribed.clone();
    asset_ids.into_iter().filter(|asset_id| seen.insert(asset_id.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_asset_ids_skips_subscribed_and_duplicates() {
        let subscribed: HashSet<AssetId> = [AssetId::from_chain(Chain::Bitcoin)].into_iter().collect();
        let result = new_asset_ids(
            &subscribed,
            vec![
                AssetId::from_chain(Chain::Bitcoin),
                AssetId::from_chain(Chain::Ethereum),
                AssetId::from_chain(Chain::Ethereum),
            ],
        );
        assert_eq!(result, vec![AssetId::from_chain(Chain::Ethereum)]);
    }

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
