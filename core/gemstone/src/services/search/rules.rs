use crate::services::chain::rules::chain_matches_query;
use crate::services::collections::unique_by;

use primitives::perpetual::{PerpetualData, PerpetualMetadata, PerpetualSearchData};
use primitives::{Asset, AssetBasic, AssetId, AssetPrice, Chain, Wallet, WalletType};

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

pub fn skips_search(scope: &GemSearchScope, query: &str) -> bool {
    *scope == GemSearchScope::All && query.is_empty()
}

pub fn stores_lists(scope: &GemSearchScope) -> bool {
    *scope == GemSearchScope::All
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

pub fn perpetual_ids(perpetuals: &[PerpetualSearchData]) -> Vec<String> {
    perpetuals.iter().map(|item| item.perpetual.id.to_string()).collect()
}

pub fn wallet_chains(wallet: &Wallet) -> Vec<Chain> {
    match wallet.wallet_type {
        WalletType::Multicoin => Vec::new(),
        WalletType::Single | WalletType::View | WalletType::PrivateKey => wallet.accounts.first().map(|account| account.chain).into_iter().collect(),
    }
}

pub fn token_chains(scope: &GemSearchScope, wallet_chains: &[Chain]) -> Vec<Chain> {
    match scope {
        GemSearchScope::All if wallet_chains.is_empty() => Chain::all(),
        GemSearchScope::All => wallet_chains.to_vec(),
        GemSearchScope::List { .. } => Vec::new(),
    }
}

pub fn api_tags(scope: &GemSearchScope) -> Vec<String> {
    match scope {
        GemSearchScope::All => Vec::new(),
        GemSearchScope::List { id } => vec![id.clone()],
    }
}

pub fn search_key(scope: &GemSearchScope, query: &str) -> String {
    let query = query.trim();
    match scope {
        GemSearchScope::List { id } if query.is_empty() => format!("tag:{id}"),
        GemSearchScope::All | GemSearchScope::List { .. } => query.to_string(),
    }
}

pub fn merge_assets(assets: Vec<AssetBasic>, tokens: Vec<AssetBasic>) -> Vec<AssetBasic> {
    unique_by(assets.into_iter().chain(tokens), |asset| asset.asset.id.clone())
}

pub fn prices(assets: &[AssetBasic]) -> Vec<AssetPrice> {
    crate::services::assets::rules::asset_prices(assets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, Asset, AssetId, AssetProperties, AssetScore, AssetType};

    fn wallet(wallet_type: WalletType, chains: &[Chain]) -> Wallet {
        Wallet {
            wallet_type,
            ..Wallet::mock_with_accounts(Account::mock_chains(chains, ""))
        }
    }

    fn asset(chain: Chain, token_id: Option<&str>) -> AssetBasic {
        let id = AssetId::from(chain, token_id.map(str::to_string));
        AssetBasic {
            asset: Asset::new(id.clone(), String::new(), String::new(), 0, AssetType::NATIVE),
            properties: AssetProperties::default(id),
            score: AssetScore::default(),
            price: None,
        }
    }

    #[test]
    fn test_wallet_chains() {
        assert!(wallet_chains(&wallet(WalletType::Multicoin, &[Chain::Bitcoin, Chain::Ethereum])).is_empty());
        assert_eq!(wallet_chains(&wallet(WalletType::Single, &[Chain::Solana])), vec![Chain::Solana]);
    }

    #[test]
    fn test_token_chains() {
        assert_eq!(token_chains(&GemSearchScope::All, &[]), Chain::all());
        assert_eq!(token_chains(&GemSearchScope::All, &[Chain::Solana]), vec![Chain::Solana]);
        assert!(token_chains(&GemSearchScope::List { id: "stocks".to_string() }, &[]).is_empty());
    }

    #[test]
    fn test_search_key() {
        let list = GemSearchScope::List { id: "stocks".to_string() };
        assert_eq!(search_key(&GemSearchScope::All, " btc "), "btc");
        assert_eq!(search_key(&list, ""), "tag:stocks");
        assert_eq!(search_key(&list, "eth"), "eth");
        assert_eq!(api_tags(&list), vec!["stocks".to_string()]);
    }

    #[test]
    fn test_merge_assets_dedupes_by_id() {
        let merged = merge_assets(
            vec![asset(Chain::Ethereum, None), asset(Chain::Ethereum, Some("0x1"))],
            vec![asset(Chain::Ethereum, Some("0x1")), asset(Chain::Solana, None)],
        );
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn test_prices_skip_assets_without_price() {
        let mut priced = asset(Chain::Ethereum, None);
        priced.price = Some(primitives::Price {
            price: 2.0,
            price_change_percentage_24h: 1.5,
            updated_at: chrono::Utc::now(),
            provider: Default::default(),
        });

        let prices = prices(&[priced, asset(Chain::Bitcoin, None)]);

        assert_eq!(prices.len(), 1);
        assert_eq!(
            (prices[0].asset_id.chain, prices[0].price, prices[0].price_change_percentage_24h),
            (Chain::Ethereum, 2.0, 1.5)
        );
    }

    #[test]
    fn test_skips_search_only_for_an_empty_query_in_the_all_scope() {
        assert!(skips_search(&GemSearchScope::All, ""));
        assert!(!skips_search(&GemSearchScope::All, "gem"));
        assert!(!skips_search(&GemSearchScope::List { id: "trending".to_string() }, ""));
    }

    #[test]
    fn test_only_the_all_scope_stores_lists() {
        assert!(stores_lists(&GemSearchScope::All));
        assert!(!stores_lists(&GemSearchScope::List { id: "trending".to_string() }));
    }

    #[test]
    fn test_assets_match_on_name_symbol_or_chain() {
        let assets = vec![Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Bitcoin)];

        assert_eq!(matching_assets(assets.clone(), "bitcoin").len(), 1);
        assert_eq!(matching_assets(assets.clone(), " ").len(), 2);
        assert!(matching_assets(assets, "dogecoin").is_empty());
    }
}
