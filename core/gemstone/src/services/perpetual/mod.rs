pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::Utc;
use primitives::currency::Currency;
use primitives::perpetual::PerpetualBalance;
use primitives::{Chain, PerpetualMarketData, PerpetualPosition, PerpetualProvider, WalletId};
use std::collections::HashMap;

use crate::config::perpetual_config::PRICES_UPDATE_INTERVAL_SECONDS;
use crate::services::preferences::GemPreferencesService;

pub use store::GemPerpetualStore;

use crate::gateway::GemGateway;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemPerpetualService {
    gateway: Arc<GemGateway>,
    price: Arc<GemPriceService>,
    store: Arc<dyn GemPerpetualStore>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemPerpetualService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, price: Arc<GemPriceService>, store: Arc<dyn GemPerpetualStore>, preferences: Arc<GemPreferencesService>) -> Self {
        Self {
            gateway,
            price,
            store,
            preferences,
        }
    }

    pub fn markets_updated_at(&self) -> Result<Option<i64>, GemServiceError> {
        self.preferences.get_perpetual_markets_updated_at()
    }

    pub async fn sync_markets(&self, chain: Chain, currency: Currency) -> Result<(), GemServiceError> {
        let data = self.gateway.get_perpetuals_data(chain).await?;
        self.store.save_perpetuals(data).await?;
        if let Some(price) = rules::collateral_price(chain) {
            self.price.update_prices(vec![price], currency).await?;
        }
        self.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp()))
    }

    pub async fn clear_markets(&self) -> Result<(), GemServiceError> {
        self.store.clear().await?;
        self.preferences.set_perpetual_markets_updated_at(None)
    }

    pub async fn set_pinned(&self, perpetual_id: String, pinned: bool) -> Result<(), GemServiceError> {
        self.store.set_pinned(vec![perpetual_id], pinned).await
    }

    pub async fn get_positions(&self, wallet_id: WalletId, chain: Chain) -> Result<Vec<PerpetualPosition>, GemServiceError> {
        self.store.get_positions(wallet_id, provider(chain)?).await
    }

    pub async fn update_positions(&self, wallet_id: WalletId, positions: Vec<PerpetualPosition>, delete_ids: Vec<String>) -> Result<(), GemServiceError> {
        self.store.update_positions(wallet_id, positions, delete_ids).await
    }

    pub async fn update_balance(&self, wallet_id: WalletId, balance: PerpetualBalance) -> Result<(), GemServiceError> {
        self.store.update_balance(wallet_id, balance).await
    }

    pub async fn update_market(&self, market: PerpetualMarketData) -> Result<(), GemServiceError> {
        self.store.update_market(market).await
    }

    pub async fn update_prices(&self, prices: HashMap<String, f64>) -> Result<(), GemServiceError> {
        let now = Utc::now().timestamp();
        if !rules::prices_outdated(self.preferences.get_perpetual_prices_updated_at()?, now, PRICES_UPDATE_INTERVAL_SECONDS) {
            return Ok(());
        }
        self.store.update_prices(prices).await?;
        self.preferences.set_perpetual_prices_updated_at(Some(now))
    }

    pub async fn sync_positions(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<(), GemServiceError> {
        let summary = self.gateway.get_positions(chain, address).await?;
        let existing_ids = self.store.get_position_ids(wallet_id.clone(), provider(chain)?).await?;
        let delete_ids = rules::stale_position_ids(existing_ids, &summary.positions);
        self.store.update_positions(wallet_id.clone(), summary.positions, delete_ids).await?;
        self.store.update_balance(wallet_id, summary.balance).await
    }
}

fn provider(chain: Chain) -> Result<PerpetualProvider, GemServiceError> {
    rules::provider(chain).ok_or_else(|| GemServiceError::Status {
        msg: format!("perpetuals unsupported on {chain}"),
    })
}
