use primitives::{BannerEvent, WalletConfiguration, WalletId};

use crate::services::banner::GemBannerKey;

pub fn multi_signature_banners(wallet_id: &WalletId, configuration: &WalletConfiguration) -> Vec<GemBannerKey> {
    configuration
        .multi_signature_accounts
        .iter()
        .map(|account| GemBannerKey {
            wallet_id: Some(wallet_id.clone()),
            asset_id: None,
            chain: Some(account.chain),
            event: BannerEvent::AccountBlockedMultiSignature,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, ChainAddress};

    #[test]
    fn test_multi_signature_banners_one_per_account() {
        let wallet_id = WalletId::Multicoin("0x1".into());
        let configuration = WalletConfiguration {
            multi_signature_accounts: vec![ChainAddress::new(Chain::Tron, "t1".into()), ChainAddress::new(Chain::Ethereum, "0x2".into())],
        };

        let banners = multi_signature_banners(&wallet_id, &configuration);

        assert_eq!(banners.len(), 2);
        assert!(
            banners
                .iter()
                .all(|key| key.wallet_id == Some(wallet_id.clone()) && key.event == BannerEvent::AccountBlockedMultiSignature)
        );
        assert_eq!(banners.iter().map(|key| key.chain).collect::<Vec<_>>(), vec![Some(Chain::Tron), Some(Chain::Ethereum)]);
        assert!(multi_signature_banners(&wallet_id, &WalletConfiguration { multi_signature_accounts: vec![] }).is_empty());
    }
}
