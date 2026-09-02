use primitives::{Asset, AssetBasic, AssetId, AssetPrice, AssetProperties, AssetScore, Chain, ConfigVersions, Wallet};

use super::model::AssetList;

use crate::models::asset::{wallet_asset_is_enabled, wallet_default_assets};
use crate::services::collections::{missing, missing_by, unique};

pub fn asset_list_versions(versions: &ConfigVersions) -> [(AssetList, i32); 3] {
    [
        (AssetList::Buy, versions.fiat_on_ramp_assets),
        (AssetList::Sell, versions.fiat_off_ramp_assets),
        (AssetList::Swap, versions.swap_assets),
    ]
}

pub fn is_asset_list_outdated(stored_version: Option<&str>, remote_version: i32) -> bool {
    stored_version != Some(remote_version.to_string().as_str())
}

pub fn asset_ids(ids: &[String]) -> Vec<AssetId> {
    ids.iter().filter_map(|id| AssetId::new(id)).collect()
}

pub fn swappable_chain_asset_ids() -> Vec<AssetId> {
    Chain::all().into_iter().filter(Chain::is_swap_supported).map(AssetId::from_chain).collect()
}

pub fn token_search_chains(chains: &[Chain]) -> Vec<Chain> {
    if chains.is_empty() { Chain::all() } else { chains.to_vec() }
}

pub fn missing_asset_ids(requested: Vec<AssetId>, existing: Vec<AssetId>) -> Vec<AssetId> {
    missing(requested, existing)
}

pub fn asset_prices(assets: &[AssetBasic]) -> Vec<AssetPrice> {
    assets
        .iter()
        .filter_map(|asset| {
            asset
                .price
                .as_ref()
                .map(|price| AssetPrice::new(asset.asset.id.clone(), price.price, price.price_change_percentage_24h, price.updated_at))
        })
        .collect()
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
    missing_by(assets, existing, |asset| asset.asset.id.clone())
}

pub fn stakeable_asset_ids() -> Vec<AssetId> {
    Chain::all().into_iter().filter(Chain::is_stake_supported).map(AssetId::from_chain).collect()
}

pub fn default_token_chain(chains: &[Chain]) -> Option<Chain> {
    chains.iter().find(|chain| **chain == Chain::Ethereum).or(chains.first()).copied()
}

pub fn token_chains(wallet: &Wallet) -> Vec<Chain> {
    let mut chains = unique(wallet.accounts.iter().map(|account| account.chain).filter(|chain| chain.default_asset_type().is_some()));
    chains.sort_by_key(|chain| std::cmp::Reverse(AssetId::from_chain(*chain).default_rank()));
    chains
}

pub fn popular_asset_ids() -> Vec<AssetId> {
    [Chain::Bitcoin, Chain::Ethereum, Chain::Solana].into_iter().map(AssetId::from_chain).collect()
}

pub fn can_open(wallet: &Wallet, asset_id: &AssetId) -> bool {
    (asset_id.is_token() || asset_id.chain.has_native_asset()) && wallet.account(asset_id.chain).is_some()
}

pub fn default_balances(wallet: &Wallet) -> (Vec<AssetId>, Vec<AssetId>) {
    unique(wallet.accounts.iter().flat_map(|account| {
        let chain = account.chain;
        let native = (chain.rank() >= 0).then(|| AssetId::from_chain(chain));
        native.into_iter().chain(wallet_default_assets(chain).into_iter().map(|asset| asset.id))
    }))
    .into_iter()
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
    fn test_token_chains_keeps_token_networks_by_rank() {
        let multicoin = wallet(WalletType::Multicoin, &[Chain::Bitcoin, Chain::Doge, Chain::Near, Chain::Xrp, Chain::Ethereum, Chain::Near]);
        assert_eq!(token_chains(&multicoin), vec![Chain::Ethereum, Chain::Xrp, Chain::Near]);
        assert!(token_chains(&wallet(WalletType::Single, &[Chain::Bitcoin])).is_empty());
    }

    #[test]
    fn test_popular_asset_ids_are_distinct_native_assets() {
        let ids = popular_asset_ids();

        assert!(!ids.is_empty());
        assert!(ids.iter().all(|id| id.is_native() && id.chain.has_native_asset()));
        assert_eq!(unique(ids.clone()).len(), ids.len());
        assert_eq!(ids.first(), Some(&AssetId::from_chain(Chain::Bitcoin)));
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
    use primitives::{Account, Chain, WalletType};

    fn wallet(wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            wallet_type,
            ..Wallet::mock_with_accounts(Account::mock_chains(chains, "address"))
        }
    }

    #[test]
    fn test_default_asset_basic_uses_default_rank_and_properties() {
        let native = default_asset_basic(Asset::from_chain(Chain::Ethereum));
        let token = default_asset_basic(Asset::new(
            AssetId::from(Chain::Ethereum, Some("0x0000000000000000000000000000000000000001".to_string())),
            String::new(),
            String::new(),
            18,
            primitives::AssetType::ERC20,
        ));

        assert_eq!(native.score.rank, Chain::Ethereum.rank());
        assert!(native.score.rank > token.score.rank);
        assert!(native.properties.is_enabled);
        assert!(native.price.is_none());
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

    #[test]
    fn test_asset_list_is_outdated_only_when_the_stored_version_differs() {
        assert!(is_asset_list_outdated(None, 7));
        assert!(is_asset_list_outdated(Some("6"), 7));
        assert!(!is_asset_list_outdated(Some("7"), 7));
    }

    #[test]
    fn test_asset_ids_skips_unparsable_identifiers() {
        let ids = asset_ids(&["bitcoin".to_string(), String::new(), "ethereum_0x1234".to_string()]);

        assert_eq!(ids, vec![AssetId::from_chain(Chain::Bitcoin), AssetId::from_token(Chain::Ethereum, "0x1234")]);
    }

    #[test]
    fn test_swappable_chain_asset_ids_only_lists_swap_supported_chains() {
        let asset_ids = swappable_chain_asset_ids();

        assert!(asset_ids.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(asset_ids.iter().all(|asset_id| asset_id.chain.is_swap_supported()));
    }

    #[test]
    fn test_token_search_chains_defaults_to_every_chain() {
        assert_eq!(token_search_chains(&[Chain::Ethereum]), vec![Chain::Ethereum]);
        assert_eq!(token_search_chains(&[]), Chain::all());
    }
}
