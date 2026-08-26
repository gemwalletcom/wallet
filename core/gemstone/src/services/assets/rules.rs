use std::collections::HashSet;

use primitives::{AssetId, Wallet};

use crate::models::asset::{wallet_asset_is_enabled, wallet_default_assets};

pub fn missing_asset_ids(requested: Vec<AssetId>, existing: Vec<AssetId>) -> Vec<AssetId> {
    let existing: HashSet<AssetId> = existing.into_iter().collect();
    let mut seen: HashSet<AssetId> = HashSet::new();
    requested
        .into_iter()
        .filter(|asset_id| !existing.contains(asset_id) && seen.insert(asset_id.clone()))
        .collect()
}

pub fn default_balances(wallet: &Wallet) -> (Vec<AssetId>, Vec<AssetId>) {
    let mut seen: HashSet<AssetId> = HashSet::new();
    wallet
        .accounts
        .iter()
        .flat_map(|account| {
            let chain = account.chain;
            let native = (chain.rank() >= 0).then(|| AssetId::from_chain(chain));
            native.into_iter().chain(wallet_default_assets(chain).into_iter().map(|asset| asset.id))
        })
        .filter(|asset_id| seen.insert(asset_id.clone()))
        .partition(|asset_id| wallet_asset_is_enabled(asset_id.clone(), wallet.wallet_type.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, Chain, WalletId, WalletSource, WalletType};

    fn wallet(wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            id: WalletId::Multicoin("0x1".to_string()),
            external_id: None,
            name: "wallet".to_string(),
            index: 0,
            wallet_type,
            accounts: chains
                .iter()
                .map(|chain| Account {
                    chain: *chain,
                    address: "address".to_string(),
                    derivation_path: String::new(),
                    extended_public_key: None,
                })
                .collect(),
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
        }
    }

    #[test]
    fn test_default_balances_by_wallet_type() {
        let (enabled, disabled) = default_balances(&wallet(WalletType::Multicoin, &[Chain::Cosmos, Chain::Ethereum, Chain::Tron]));
        assert!(disabled.contains(&AssetId::from_chain(Chain::Cosmos)));
        assert!(enabled.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(wallet_default_assets(Chain::Tron).iter().all(|asset| enabled.contains(&asset.id)));

        let (enabled, disabled) = default_balances(&wallet(WalletType::Single, &[Chain::Cosmos]));
        assert_eq!(enabled, vec![AssetId::from_chain(Chain::Cosmos)]);
        assert!(disabled.is_empty());

        let (enabled, _) = default_balances(&wallet(WalletType::Single, &[Chain::Tempo]));
        assert!(!enabled.contains(&AssetId::from_chain(Chain::Tempo)));
        assert!(wallet_default_assets(Chain::Tempo).iter().all(|asset| enabled.contains(&asset.id)));
    }

    #[test]
    fn test_missing_asset_ids_drops_known_and_duplicate_ids() {
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let ethereum = AssetId::from_chain(Chain::Ethereum);

        let missing = missing_asset_ids(vec![bitcoin.clone(), ethereum.clone(), ethereum.clone()], vec![bitcoin]);

        assert_eq!(missing, vec![ethereum]);
    }
}
