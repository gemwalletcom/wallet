use std::collections::HashSet;

use primitives::{AssetId, BannerEvent, WalletConfiguration, WalletId};

use crate::services::banner::GemBannerKey;

pub fn externally_controlled_banners(wallet_id: &WalletId, configuration: &WalletConfiguration) -> Vec<GemBannerKey> {
    let mut seen = HashSet::new();
    configuration
        .multi_signature_accounts
        .iter()
        .chain(&configuration.externally_controlled_accounts)
        .filter(|account| seen.insert(*account))
        .map(|account| GemBannerKey {
            wallet_id: Some(wallet_id.clone()),
            asset_id: Some(AssetId::from_chain(account.chain)),
            event: BannerEvent::AccountBlockedMultiSignature,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, ChainAddress};

    #[test]
    fn test_externally_controlled_banners_one_per_account_from_either_field() {
        let wallet_id = WalletId::Multicoin("0x1".into());
        let accounts = vec![ChainAddress::new(Chain::Tron, "t1".into()), ChainAddress::new(Chain::Solana, "s1".into())];
        let current_api = WalletConfiguration {
            multi_signature_accounts: accounts.clone(),
            externally_controlled_accounts: accounts.clone(),
        };
        let previous_api = WalletConfiguration {
            multi_signature_accounts: accounts,
            externally_controlled_accounts: vec![],
        };
        let empty = WalletConfiguration {
            multi_signature_accounts: vec![],
            externally_controlled_accounts: vec![],
        };

        let banners = externally_controlled_banners(&wallet_id, &current_api);

        assert_eq!(
            banners,
            vec![
                GemBannerKey {
                    wallet_id: Some(wallet_id.clone()),
                    asset_id: Some(AssetId::from_chain(Chain::Tron)),
                    event: BannerEvent::AccountBlockedMultiSignature,
                },
                GemBannerKey {
                    wallet_id: Some(wallet_id.clone()),
                    asset_id: Some(AssetId::from_chain(Chain::Solana)),
                    event: BannerEvent::AccountBlockedMultiSignature,
                },
            ]
        );
        assert_eq!(externally_controlled_banners(&wallet_id, &previous_api), banners);
        assert!(externally_controlled_banners(&wallet_id, &empty).is_empty());
    }
}
