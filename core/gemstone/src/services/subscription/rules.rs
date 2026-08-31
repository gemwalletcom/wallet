use std::collections::{BTreeSet, HashMap};

use primitives::{AddressChains, Chain, Wallet, WalletSubscription, WalletSubscriptionChains};

#[derive(Debug, Default, PartialEq)]
pub struct SubscriptionChanges {
    pub to_add: Vec<WalletSubscription>,
    pub to_delete: Vec<WalletSubscriptionChains>,
}

impl SubscriptionChanges {
    pub fn is_empty(&self) -> bool {
        self.to_add.is_empty() && self.to_delete.is_empty()
    }
}

pub fn wallet_subscriptions(wallets: &[Wallet]) -> Vec<WalletSubscription> {
    wallets
        .iter()
        .map(|wallet| WalletSubscription {
            wallet_id: wallet.id.clone(),
            source: Some(wallet.source.clone()),
            subscriptions: wallet.address_chains(),
        })
        .collect()
}

pub fn subscription_changes(local: Vec<WalletSubscription>, remote: Vec<WalletSubscriptionChains>) -> SubscriptionChanges {
    let remote_chains: HashMap<String, BTreeSet<Chain>> = remote.iter().map(|wallet| (wallet.wallet_id.id(), wallet.chains.iter().copied().collect())).collect();
    let local_chains: HashMap<String, BTreeSet<Chain>> = local
        .iter()
        .map(|wallet| {
            let chains = wallet.subscriptions.iter().flat_map(|address| address.chains.iter().copied()).collect();
            (wallet.wallet_id.id(), chains)
        })
        .collect();

    let to_add = local
        .into_iter()
        .filter_map(|wallet| {
            let known = remote_chains.get(&wallet.wallet_id.id());
            let subscriptions: Vec<AddressChains> = wallet
                .subscriptions
                .into_iter()
                .filter_map(|address| {
                    let chains: Vec<Chain> = address.chains.into_iter().filter(|chain| !known.is_some_and(|known| known.contains(chain))).collect();
                    (!chains.is_empty()).then(|| AddressChains::new(address.address, chains))
                })
                .collect();
            (!subscriptions.is_empty()).then_some(WalletSubscription {
                wallet_id: wallet.wallet_id,
                source: wallet.source,
                subscriptions,
            })
        })
        .collect();

    let to_delete = remote
        .into_iter()
        .filter_map(|wallet| {
            let Some(local) = local_chains.get(&wallet.wallet_id.id()) else {
                return Some(wallet);
            };
            let chains: BTreeSet<Chain> = wallet.chains.into_iter().filter(|chain| !local.contains(chain)).collect();
            (!chains.is_empty()).then(|| WalletSubscriptionChains {
                wallet_id: wallet.wallet_id,
                chains: chains.into_iter().collect(),
            })
        })
        .collect();

    SubscriptionChanges { to_add, to_delete }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, WalletId, WalletSource, WalletType};

    fn local(wallet_id: &str, subscriptions: Vec<(&str, Vec<Chain>)>) -> WalletSubscription {
        WalletSubscription {
            wallet_id: WalletId::Multicoin(wallet_id.into()),
            source: Some(WalletSource::Import),
            subscriptions: subscriptions.into_iter().map(|(address, chains)| AddressChains::new(address.into(), chains)).collect(),
        }
    }

    fn remote(wallet_id: &str, chains: Vec<Chain>) -> WalletSubscriptionChains {
        WalletSubscriptionChains {
            wallet_id: WalletId::Multicoin(wallet_id.into()),
            chains,
        }
    }

    #[test]
    fn test_add_new_chain_to_existing_wallet() {
        let changes = subscription_changes(
            vec![local("wallet1", vec![("btc1", vec![Chain::Bitcoin]), ("eth1", vec![Chain::Ethereum])])],
            vec![remote("wallet1", vec![Chain::Bitcoin])],
        );

        assert_eq!(changes.to_add.len(), 1);
        assert_eq!(changes.to_add[0].subscriptions, vec![AddressChains::new("eth1".into(), vec![Chain::Ethereum])]);
        assert!(changes.to_delete.is_empty());
    }

    #[test]
    fn test_remove_chain_from_existing_wallet() {
        let changes = subscription_changes(
            vec![local("wallet1", vec![("btc1", vec![Chain::Bitcoin])])],
            vec![remote("wallet1", vec![Chain::Bitcoin, Chain::Ethereum])],
        );

        assert!(changes.to_add.is_empty());
        assert_eq!(changes.to_delete, vec![remote("wallet1", vec![Chain::Ethereum])]);
    }

    #[test]
    fn test_delete_entire_wallet_and_add_new_wallet() {
        let deleted = subscription_changes(vec![], vec![remote("wallet1", vec![Chain::Bitcoin, Chain::Ethereum])]);
        assert!(deleted.to_add.is_empty());
        assert_eq!(deleted.to_delete, vec![remote("wallet1", vec![Chain::Bitcoin, Chain::Ethereum])]);

        let added = subscription_changes(vec![local("wallet2", vec![("sol1", vec![Chain::Solana])])], vec![]);
        assert_eq!(added.to_add.len(), 1);
        assert_eq!(added.to_add[0].wallet_id, WalletId::Multicoin("wallet2".into()));
        assert!(added.to_delete.is_empty());
    }

    #[test]
    fn test_no_changes_when_in_sync() {
        let changes = subscription_changes(
            vec![local("wallet1", vec![("btc1", vec![Chain::Bitcoin]), ("eth1", vec![Chain::Ethereum])])],
            vec![remote("wallet1", vec![Chain::Bitcoin, Chain::Ethereum])],
        );

        assert!(changes.is_empty());
    }

    #[test]
    fn test_multiple_wallets_with_changes() {
        let changes = subscription_changes(
            vec![
                local("wallet1", vec![("btc1", vec![Chain::Bitcoin])]),
                local("wallet2", vec![("eth1", vec![Chain::Ethereum]), ("poly1", vec![Chain::Polygon])]),
            ],
            vec![remote("wallet1", vec![Chain::Bitcoin, Chain::Ethereum]), remote("wallet3", vec![Chain::Solana])],
        );

        assert_eq!(
            changes.to_add.iter().map(|wallet| wallet.wallet_id.clone()).collect::<Vec<_>>(),
            vec![WalletId::Multicoin("wallet2".into())]
        );
        assert_eq!(changes.to_add[0].subscriptions.len(), 2);
        assert_eq!(changes.to_delete, vec![remote("wallet1", vec![Chain::Ethereum]), remote("wallet3", vec![Chain::Solana])]);
    }

    #[test]
    fn test_wallet_subscriptions_group_chains_by_address() {
        let account = |chain: Chain, address: &str| Account {
            chain,
            address: address.into(),
            derivation_path: "".into(),
            extended_public_key: None,
        };
        let wallet = Wallet {
            id: WalletId::Multicoin("wallet1".into()),
            external_id: None,
            name: "wallet".into(),
            index: 0,
            wallet_type: WalletType::Multicoin,
            accounts: vec![account(Chain::Ethereum, "0xevm"), account(Chain::Polygon, "0xevm"), account(Chain::Bitcoin, "bc1")],
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        };

        let subscriptions = wallet_subscriptions(&[wallet]);

        assert_eq!(
            subscriptions[0].subscriptions,
            vec![
                AddressChains::new("0xevm".into(), vec![Chain::Ethereum, Chain::Polygon]),
                AddressChains::new("bc1".into(), vec![Chain::Bitcoin]),
            ]
        );
    }
}
