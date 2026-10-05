use crate::services::chain::rules::chain_matches_query;

use primitives::perpetual::{PerpetualData, PerpetualMetadata, PerpetualSearchData};
use primitives::{Asset, AssetBasic, AssetId, Chain, PerpetualId, Wallet, WalletType};

use super::model::GemSearchScope;

pub fn matching_assets(assets: Vec<Asset>, query: &str) -> Vec<Asset> {
    let trimmed = query.trim().to_lowercase();
    if trimmed.is_empty() {
        return assets;
    }
    assets
        .into_iter()
        .filter(|asset| asset.name.to_lowercase().contains(&trimmed) || asset.symbol.to_lowercase().contains(&trimmed) || chain_matches_query(asset.chain(), &trimmed))
        .collect()
}

impl GemSearchScope {
    pub(super) fn skips_search(&self, query: &str) -> bool {
        match self {
            Self::All => query.is_empty(),
            Self::List { .. } => false,
        }
    }

    pub(super) fn stores_lists(&self) -> bool {
        match self {
            Self::All => true,
            Self::List { .. } => false,
        }
    }

    pub(super) fn token_chains(&self, wallet_chains: &[Chain]) -> Vec<Chain> {
        match self {
            Self::All if wallet_chains.is_empty() => Chain::all(),
            Self::All => wallet_chains.to_vec(),
            Self::List { .. } => Vec::new(),
        }
    }

    pub(crate) fn includes_perpetuals(&self) -> bool {
        match self {
            Self::All => false,
            Self::List { .. } => true,
        }
    }

    pub(super) fn api_tags(&self) -> Vec<String> {
        match self {
            Self::All => Vec::new(),
            Self::List { id } => vec![id.clone()],
        }
    }
}

pub fn asset_ids(assets: &[AssetBasic]) -> Vec<AssetId> {
    assets.iter().map(|asset| asset.asset.id.clone()).collect()
}

pub fn perpetual_data(perpetuals: &[PerpetualSearchData]) -> Vec<PerpetualData> {
    perpetuals
        .iter()
        .map(|item| PerpetualData {
            perpetual: item.perpetual.clone(),
            asset: item.asset.clone(),
            metadata: PerpetualMetadata { is_pinned: false },
        })
        .collect()
}

pub fn perpetual_ids(perpetuals: &[PerpetualSearchData]) -> Vec<PerpetualId> {
    perpetuals.iter().map(|item| item.perpetual.id.clone()).collect()
}

pub fn wallet_chains(wallet: &Wallet) -> Vec<Chain> {
    match wallet.wallet_type {
        WalletType::Multicoin => Vec::new(),
        WalletType::Single | WalletType::View | WalletType::PrivateKey => wallet.accounts.first().map(|account| account.chain).into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_chains() {
        assert!(wallet_chains(&Wallet::mock_with_type(WalletType::Multicoin, &[Chain::Bitcoin, Chain::Ethereum])).is_empty());
        assert_eq!(wallet_chains(&Wallet::mock_with_type(WalletType::Single, &[Chain::Solana])), vec![Chain::Solana]);
    }

    #[test]
    fn test_token_chains() {
        assert_eq!(GemSearchScope::All.token_chains(&[]), Chain::all());
        assert_eq!(GemSearchScope::All.token_chains(&[Chain::Solana]), vec![Chain::Solana]);
        assert!(GemSearchScope::List { id: "stocks".to_string() }.token_chains(&[]).is_empty());
    }

    #[test]
    fn test_skips_search_only_for_an_empty_query_in_the_all_scope() {
        assert!(GemSearchScope::All.skips_search(""));
        assert!(!GemSearchScope::All.skips_search("gem"));
        assert!(!GemSearchScope::List { id: "trending".to_string() }.skips_search(""));
    }

    #[test]
    fn test_only_the_all_scope_stores_lists() {
        assert!(GemSearchScope::All.stores_lists());
        assert!(!GemSearchScope::List { id: "trending".to_string() }.stores_lists());
    }

    #[test]
    fn test_only_a_list_includes_perpetuals_in_its_results() {
        assert!(GemSearchScope::List { id: "trending".to_string() }.includes_perpetuals());
        assert!(!GemSearchScope::All.includes_perpetuals(), "the full results of a typed search are assets; its perpetuals open their own screen");
    }

    #[test]
    fn test_assets_match_on_name_symbol_or_chain() {
        let assets = vec![Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Bitcoin)];

        assert_eq!(matching_assets(assets.clone(), "bitcoin").len(), 1);
        assert_eq!(matching_assets(assets.clone(), " ").len(), 2);
        assert!(matching_assets(assets, "dogecoin").is_empty());
    }
}
