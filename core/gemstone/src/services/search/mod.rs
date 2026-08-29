pub mod model;
pub mod rules;
pub mod store;

use std::sync::Arc;

use primitives::currency::Currency;
use primitives::perpetual::PerpetualSearchData;
use primitives::{Asset, AssetBasic, Wallet};

pub use model::GemSearchScope;
pub use store::GemSearchStore;

use crate::services::assets::{GemAssetStore, GemAssetsService};
use crate::services::error::GemServiceError;
use crate::services::perpetual::{GemPerpetualStore, rules as perpetual_rules};
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemSearchService {
    assets: Arc<GemAssetsService>,
    asset_store: Arc<dyn GemAssetStore>,
    price: Arc<GemPriceService>,
    perpetual_store: Arc<dyn GemPerpetualStore>,
    store: Arc<dyn GemSearchStore>,
}

#[uniffi::export]
impl GemSearchService {
    #[uniffi::constructor]
    pub fn new(
        assets: Arc<GemAssetsService>,
        asset_store: Arc<dyn GemAssetStore>,
        price: Arc<GemPriceService>,
        perpetual_store: Arc<dyn GemPerpetualStore>,
        store: Arc<dyn GemSearchStore>,
    ) -> Self {
        Self {
            assets,
            asset_store,
            price,
            perpetual_store,
            store,
        }
    }

    pub async fn search(&self, wallet: Wallet, query: String, scope: GemSearchScope, currency: Currency) -> Result<bool, GemServiceError> {
        let query = query.trim().to_string();
        if rules::skips_search(&scope, &query) {
            return Ok(false);
        }
        let wallet_chains = rules::wallet_chains(&wallet);
        let (response, tokens) = futures::join!(
            self.assets.search(query.clone(), wallet_chains.clone(), rules::api_tags(&scope)),
            self.assets.search_tokens(query.clone(), rules::token_chains(&scope, &wallet_chains)),
        );
        let response = response?;
        let assets = rules::merge_assets(response.assets, tokens);
        let key = rules::search_key(&scope, &query);
        self.save_assets(&wallet, &assets, currency, &key).await?;
        self.save_perpetuals(&response.perpetuals, &key).await?;
        if rules::stores_lists(&scope) {
            self.store.set_lists(key, response.lists).await?;
        }
        Ok(!assets.is_empty() || !response.perpetuals.is_empty())
    }

    pub async fn search_assets(&self, wallet: Wallet, query: String, currency: Currency) -> Result<Vec<AssetBasic>, GemServiceError> {
        let assets = self.assets.search_assets_and_tokens(query.clone(), rules::wallet_chains(&wallet)).await?;
        self.save_assets(&wallet, &assets, currency, &rules::search_key(&GemSearchScope::All, &query)).await?;
        Ok(assets)
    }
}

impl GemSearchService {
    async fn save_assets(&self, wallet: &Wallet, assets: &[AssetBasic], currency: Currency, key: &str) -> Result<(), GemServiceError> {
        let asset_ids = rules::asset_ids(assets);
        self.asset_store.save_assets(assets.to_vec()).await?;
        self.price.update_prices(rules::prices(assets), currency).await?;
        self.assets.add_missing_balances(wallet.id.clone(), asset_ids.clone()).await?;
        self.store.set_assets(key.to_string(), asset_ids).await
    }

    async fn save_perpetuals(&self, perpetuals: &[PerpetualSearchData], key: &str) -> Result<(), GemServiceError> {
        let data = rules::perpetual_data(perpetuals);
        self.asset_store.save_assets(perpetual_rules::perpetual_asset_basics(&data)).await?;
        self.perpetual_store.save_perpetuals(data).await?;
        self.store.set_perpetuals(key.to_string(), rules::perpetual_ids(perpetuals)).await
    }
}

#[uniffi::export]
pub fn search_matching_assets(assets: Vec<Asset>, query: String) -> Vec<Asset> {
    rules::matching_assets(assets, &query)
}
