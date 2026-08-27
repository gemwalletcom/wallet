use std::collections::HashSet;

use primitives::{Asset, AssetBasic, AssetId, AssetProperties, AssetScore, Chain, Wallet};

use crate::models::asset::{wallet_asset_is_enabled, wallet_default_assets};
use crate::services::collections::missing;

pub fn missing_asset_ids(requested: Vec<AssetId>, existing: Vec<AssetId>) -> Vec<AssetId> {
    missing(requested, existing)
}

pub fn default_asset_basic(asset: Asset) -> AssetBasic {
    let asset_id = asset.id.clone();
    AssetBasic::new(asset, AssetProperties::default(asset_id.clone()), AssetScore::new(asset_id.default_rank()))
}

pub fn default_assets() -> Vec<AssetBasic> {
    Chain::all()
        .into_iter()
        .flat_map(|chain| std::iter::once(Asset::from_chain(chain)).chain(wallet_default_assets(chain)))
        .map(default_asset_basic)
        .collect()
}

pub fn missing_assets(assets: Vec<AssetBasic>, existing: Vec<AssetId>) -> Vec<AssetBasic> {
    let existing: HashSet<AssetId> = existing.into_iter().collect();
    assets.into_iter().filter(|asset| !existing.contains(&asset.asset.id)).collect()
}

pub fn stakeable_asset_ids() -> Vec<AssetId> {
    Chain::all().into_iter().filter(Chain::is_stake_supported).map(AssetId::from_chain).collect()
}

pub fn default_token_chain(chains: &[Chain]) -> Option<Chain> {
    chains.iter().find(|chain| **chain == Chain::Ethereum).or(chains.first()).copied()
}

pub fn popular_asset_ids() -> Vec<AssetId> {
    [Chain::Bitcoin, Chain::Ethereum, Chain::Solana].into_iter().map(AssetId::from_chain).collect()
}

pub fn can_open(wallet: &Wallet, asset_id: &AssetId) -> bool {
    (asset_id.is_token() || asset_id.chain.has_native_asset()) && wallet.account(asset_id.chain).is_some()
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
    #[test]
    fn test_default_token_chain_prefers_ethereum_then_first() {
        assert_eq!(default_token_chain(&[Chain::Solana, Chain::Ethereum]), Some(Chain::Ethereum));
        assert_eq!(default_token_chain(&[Chain::Solana, Chain::Tron]), Some(Chain::Solana));
        assert_eq!(default_token_chain(&[]), None);
    }

    #[test]
    fn test_popular_asset_ids_are_native_majors_in_order() {
        assert_eq!(
            popular_asset_ids(),
            vec![
                AssetId::from_chain(Chain::Bitcoin),
                AssetId::from_chain(Chain::Ethereum),
                AssetId::from_chain(Chain::Solana)
            ]
        );
    }

    #[test]
    fn test_can_open_requires_account_and_native_asset() {
        let wallet = wallet(WalletType::Multicoin, &[Chain::Ethereum, Chain::Tempo]);
        assert!(can_open(&wallet, &AssetId::from_chain(Chain::Ethereum)));
        assert!(can_open(
            &wallet,
            &AssetId::from(Chain::Ethereum, Some("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string()))
        ));
        assert!(!can_open(&wallet, &AssetId::from_chain(Chain::Bitcoin)));
        assert!(!can_open(&wallet, &AssetId::from_chain(Chain::Tempo)));
        assert!(can_open(
            &wallet,
            &AssetId::from(Chain::Tempo, Some("0x20c000000000000000000000c48d6a3bd5b7b0c2".to_string()))
        ));
    }

    #[test]
    fn test_default_assets_and_missing() {
        let assets = default_assets();
        let bitcoin = AssetId::from_chain(Chain::Bitcoin);
        let tron_usdt = wallet_default_assets(Chain::Tron)[0].id.clone();
        assert!(assets.iter().any(|asset| asset.asset.id == bitcoin));
        assert!(assets.iter().any(|asset| asset.asset.id == tron_usdt));

        let missing = missing_assets(assets.clone(), vec![bitcoin.clone()]);
        assert_eq!(missing.len(), assets.len() - 1);
        assert!(!missing.iter().any(|asset| asset.asset.id == bitcoin));
        assert!(stakeable_asset_ids().contains(&AssetId::from_chain(Chain::Cosmos)));
        assert!(!stakeable_asset_ids().contains(&bitcoin));
    }
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
    fn test_default_asset_basic_uses_default_rank_and_properties() {
        let basic = default_asset_basic(Asset::from_chain(Chain::Ethereum));
        assert_eq!(basic.score.rank, AssetId::from_chain(Chain::Ethereum).default_rank());
        assert!(basic.properties.is_enabled);
        assert!(basic.price.is_none());
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
