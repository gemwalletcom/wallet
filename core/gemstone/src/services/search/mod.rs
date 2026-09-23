pub mod model;
pub mod rules;
pub mod store;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::perpetual::PerpetualSearchData;
use primitives::{AssetBasic, Wallet};

pub use model::GemSearchScope;
pub use store::GemSearchStore;

use crate::services::assets::{GemAssetsService, rules as assets_rules};
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemSearchService {
    assets: Arc<GemAssetsService>,
    price: Arc<GemPriceService>,
    perpetuals: Arc<GemPerpetualService>,
    store: Arc<dyn GemSearchStore>,
}

#[uniffi::export]
impl GemSearchService {
    #[uniffi::constructor]
    pub fn new(assets: Arc<GemAssetsService>, price: Arc<GemPriceService>, perpetuals: Arc<GemPerpetualService>, store: Arc<dyn GemSearchStore>) -> Self {
        Self { assets, price, perpetuals, store }
    }
}

impl GemSearchService {
    pub async fn search(&self, wallet: Wallet, query: String, scope: GemSearchScope, currency: Currency) -> Result<bool, GemServiceError> {
        let query = query.trim().to_string();
        if scope.skips_search(&query) {
            return Ok(false);
        }
        let wallet_chains = rules::wallet_chains(&wallet);
        let (response, tokens) = futures::join!(
            self.assets.search(query.clone(), wallet_chains.clone(), scope.api_tags()),
            self.assets.search_tokens(query.clone(), scope.token_chains(&wallet_chains)),
        );
        let response = response?;
        let assets = assets_rules::merge_assets(response.assets, tokens);
        let key = scope.search_key(&query);
        self.save_assets(&wallet, &assets, currency, &key).await?;
        self.save_perpetuals(&response.perpetuals, &key).await?;
        if scope.stores_lists() {
            self.store.set_lists(key, response.lists).await?;
        }
        Ok(!assets.is_empty() || !response.perpetuals.is_empty())
    }

    pub async fn search_assets(&self, wallet: Wallet, query: String, currency: Currency) -> Result<Vec<AssetBasic>, GemServiceError> {
        let assets = self.assets.search_assets_and_tokens(query.clone(), rules::wallet_chains(&wallet)).await?;
        self.save_assets(&wallet, &assets, currency, &GemSearchScope::All.search_key(&query)).await?;
        Ok(assets)
    }

    async fn save_assets(&self, wallet: &Wallet, assets: &[AssetBasic], currency: Currency, key: &str) -> Result<(), GemServiceError> {
        let asset_ids = rules::asset_ids(assets);
        self.assets.save_assets(assets.to_vec()).await?;
        self.price.update_prices(rules::prices(assets), currency).await?;
        self.assets.add_missing_balances(wallet.id.clone(), asset_ids.clone()).await?;
        self.store.set_assets(key.to_string(), asset_ids).await
    }

    async fn save_perpetuals(&self, perpetuals: &[PerpetualSearchData], key: &str) -> Result<(), GemServiceError> {
        self.perpetuals.save_markets(rules::perpetual_data(perpetuals)).await?;
        self.store.set_perpetuals(key.to_string(), rules::perpetual_ids(perpetuals)).await
    }
}
