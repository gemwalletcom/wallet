pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::perpetual::PerpetualSearchData;
use primitives::{AssetBasic, Wallet};

pub use model::GemSearchScope;
pub use store::GemSearchStore;

use crate::api::{GemApiClient, GemApiError};
use crate::services::assets::{GemAssetsService, rules as assets_rules};
use crate::services::balance::GemBalanceService;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemSearchService {
    api: Arc<GemApiClient>,
    assets: Arc<GemAssetsService>,
    balance: Arc<GemBalanceService>,
    price: Arc<GemPriceService>,
    perpetuals: Arc<GemPerpetualService>,
    store: Arc<dyn GemSearchStore>,
}

#[uniffi::export]
impl GemSearchService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, assets: Arc<GemAssetsService>, balance: Arc<GemBalanceService>, price: Arc<GemPriceService>, perpetuals: Arc<GemPerpetualService>, store: Arc<dyn GemSearchStore>) -> Self {
        Self {
            api,
            assets,
            balance,
            price,
            perpetuals,
            store,
        }
    }
}

impl GemSearchService {
    pub async fn search(&self, wallet: Wallet, query: String, scope: GemSearchScope) -> Result<bool, GemServiceError> {
        let query = query.trim().to_string();
        if scope.skips_search(&query) {
            return Ok(false);
        }
        let wallet_chains = rules::wallet_chains(&wallet);
        let (response, tokens) = futures::join!(
            self.api.client.get_search(query.clone(), wallet_chains.clone(), scope.api_tags()),
            self.assets.search_tokens(query.clone(), scope.token_chains(&wallet_chains)),
        );
        let response = response.map_err(GemApiError::from)?;
        let assets = assets_rules::merge_assets(response.assets, tokens);
        let key = scope.search_key(query);
        self.save_assets(&wallet, &assets, &key).await?;
        self.save_perpetuals(&response.perpetuals, &key).await?;
        if scope.stores_lists() {
            self.store.set_lists(key, response.lists).await?;
        }
        Ok(!assets.is_empty() || !response.perpetuals.is_empty())
    }

    pub async fn search_assets(&self, wallet: Wallet, query: String) -> Result<Vec<AssetBasic>, GemServiceError> {
        let scope = GemSearchScope::All;
        let wallet_chains = rules::wallet_chains(&wallet);
        let (assets, tokens) = futures::join!(
            self.api.client.get_search_assets(query.clone(), wallet_chains.clone()),
            self.assets.search_tokens(query.clone(), scope.token_chains(&wallet_chains)),
        );
        let assets = assets_rules::merge_assets(assets.map_err(GemApiError::from)?, tokens);
        self.save_assets(&wallet, &assets, &scope.search_key(query)).await?;
        Ok(assets)
    }

    async fn save_assets(&self, wallet: &Wallet, assets: &[AssetBasic], key: &str) -> Result<(), GemServiceError> {
        let asset_ids = rules::asset_ids(assets);
        self.assets.save_assets(assets.to_vec()).await?;
        self.price.update_prices(rules::prices(assets)).await?;
        self.balance.add_missing_balances(wallet.id.clone(), asset_ids.clone()).await?;
        self.store.set_assets(key.to_string(), asset_ids).await
    }

    async fn save_perpetuals(&self, perpetuals: &[PerpetualSearchData], key: &str) -> Result<(), GemServiceError> {
        self.perpetuals.save_markets(rules::perpetual_data(perpetuals)).await?;
        self.store.set_perpetuals(key.to_string(), rules::perpetual_ids(perpetuals)).await
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::asset_constants::ETHEREUM_USDT_ASSET_ID;
    use primitives::{AssetList, Chain};

    use super::testkit::SearchTestkit;
    use super::*;
    use crate::services::node::rules::preferred_chain_node;

    const USDT_SEARCH_RESPONSE: &str = r#"{
        "assets": [{
            "asset": {"id": "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", "name": "Tether", "symbol": "USDT", "decimals": 6, "type": "ERC20"},
            "properties": {"isEnabled": true, "isBuyable": true, "isSellable": true, "isSwapable": true, "isStakeable": false, "isEarnable": false, "hasImage": true, "hasPrice": true},
            "score": {"rank": 34, "type": "low"},
            "price": {"price": 0.9998, "priceChangePercentage24h": -0.01, "updatedAt": "2026-09-27T22:44:30Z"}
        }],
        "perpetuals": [],
        "nfts": [],
        "lists": [{"id": "stablecoins", "name": "Stablecoins", "count": 12}]
    }"#;

    const ETH_SEARCH_ASSETS: &str = r#"[{
        "asset": {"id": "ethereum", "name": "Ethereum", "symbol": "ETH", "decimals": 18, "type": "NATIVE"},
        "properties": {"isEnabled": true, "isBuyable": true, "isSellable": true, "isSwapable": true, "isStakeable": false, "isEarnable": false, "hasImage": true, "hasPrice": true},
        "score": {"rank": 100, "type": "high"}
    }]"#;

    #[test]
    fn test_search_skips_an_empty_query_in_the_all_scope() {
        block_on(async {
            let testkit = SearchTestkit::with_status(503);

            assert!(!testkit.service.search(testkit.wallet.clone(), " ".to_string(), GemSearchScope::All).await.unwrap());
            assert!(testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_search_saves_assets_perpetuals_and_lists_under_the_query() {
        block_on(async {
            let testkit = SearchTestkit::with_bodies(&[("/v1/search?", USDT_SEARCH_RESPONSE)]);
            let usdt = ETHEREUM_USDT_ASSET_ID.clone();

            assert!(testkit.service.search(testkit.wallet.clone(), " usdt ".to_string(), GemSearchScope::All).await.unwrap());

            assert_eq!(testkit.provider.requested_paths(), vec!["/v1/search?query=usdt&chains=&tags=".to_string(), preferred_chain_node(Chain::Near, None).url]);
            assert_eq!(testkit.asset_store.asset_writes.lock().unwrap().len(), 1);
            assert_eq!(testkit.prices.prices.lock().unwrap().len(), 1);
            assert_eq!(*testkit.balances.added_balances.lock().unwrap(), vec![(testkit.wallet.id.clone(), vec![usdt.clone()], false)]);
            assert_eq!(*testkit.store.assets.lock().unwrap(), vec![("usdt".to_string(), vec![usdt])]);
            assert_eq!(*testkit.store.perpetuals.lock().unwrap(), vec![("usdt".to_string(), vec![])]);
            assert_eq!(*testkit.store.lists.lock().unwrap(), vec![("usdt".to_string(), vec![AssetList::mock(12)])]);
        })
    }

    #[test]
    fn test_a_list_scope_asks_with_its_tag_and_leaves_the_lists_alone() {
        block_on(async {
            let testkit = SearchTestkit::with_bodies(&[("/v1/search?", USDT_SEARCH_RESPONSE)]);
            let stocks = GemSearchScope::List { id: "stocks".to_string() };

            assert!(testkit.service.search(testkit.wallet.clone(), String::new(), stocks).await.unwrap());

            assert_eq!(testkit.provider.requested_paths(), vec!["/v1/search?query=&chains=&tags=stocks".to_string()]);
            assert_eq!(*testkit.store.assets.lock().unwrap(), vec![("tag:stocks".to_string(), vec![ETHEREUM_USDT_ASSET_ID.clone()])]);
            assert!(testkit.store.lists.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_search_assets_returns_what_it_saved() {
        block_on(async {
            let testkit = SearchTestkit::with_bodies(&[("/v1/assets/search?", ETH_SEARCH_ASSETS)]);
            let ethereum = Chain::Ethereum.as_asset_id();

            let assets = testkit.service.search_assets(testkit.wallet.clone(), "eth".to_string()).await.unwrap();

            assert_eq!(assets.iter().map(|basic| basic.asset.id.clone()).collect::<Vec<_>>(), vec![ethereum.clone()]);
            assert_eq!(testkit.provider.requested_paths(), vec!["/v1/assets/search?query=eth&chains=".to_string(), preferred_chain_node(Chain::Near, None).url]);
            assert_eq!(*testkit.store.assets.lock().unwrap(), vec![("eth".to_string(), vec![ethereum])]);
            assert!(testkit.store.lists.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_a_failed_backend_search_saves_nothing() {
        block_on(async {
            let testkit = SearchTestkit::with_status(503);

            assert!(testkit.service.search(testkit.wallet.clone(), "usdt".to_string(), GemSearchScope::All).await.is_err());
            assert!(testkit.service.search_assets(testkit.wallet.clone(), "usdt".to_string()).await.is_err());
            assert!(testkit.asset_store.asset_writes.lock().unwrap().is_empty());
            assert!(testkit.store.assets.lock().unwrap().is_empty());
        })
    }
}
