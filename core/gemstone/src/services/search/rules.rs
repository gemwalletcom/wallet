use crate::services::collections::unique_by;

use primitives::{AssetBasic, AssetPrice, Chain, Wallet, WalletType};

use super::model::GemSearchScope;

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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Account, Asset, AssetId, AssetProperties, AssetScore, AssetType, WalletId, WalletSource};

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
                    address: String::new(),
                    derivation_path: String::new(),
                    extended_public_key: None,
                })
                .collect(),
            is_pinned: false,
            image_url: None,
            source: WalletSource::Import,
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
}
