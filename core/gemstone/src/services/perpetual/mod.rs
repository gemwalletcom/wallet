pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::Utc;
use gem_hypercore::models::websocket::HyperliquidSocketMessage;
use gem_hypercore::provider::websocket_mapper::{diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data};
use primitives::perpetual::PerpetualBalance;
use primitives::portfolio::PerpetualPortfolio;
use primitives::{AssetId, Chain, ChartPeriod, PerpetualAccountMode, PerpetualProvider, WalletId};
use std::collections::HashMap;

use crate::config::perpetual_config::PRICES_UPDATE_INTERVAL_SECONDS;
use crate::services::preferences::GemPreferencesService;

pub use model::GemPerpetualSocketUpdate;
pub use store::GemPerpetualStore;

use crate::gateway::GemGateway;
use crate::models::perpetual::GemChartCandleStick;
use crate::services::balance::GemBalanceService;
use crate::services::price::GemPriceService;
use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemPerpetualService {
    gateway: Arc<GemGateway>,
    price: Arc<GemPriceService>,
    store: Arc<dyn GemPerpetualStore>,
    preferences: Arc<GemPreferencesService>,
    balance: Arc<GemBalanceService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemPerpetualService {
    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        price: Arc<GemPriceService>,
        store: Arc<dyn GemPerpetualStore>,
        preferences: Arc<GemPreferencesService>,
        balance: Arc<GemBalanceService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            gateway,
            price,
            store,
            preferences,
            balance,
            wallet_preferences,
        }
    }

    pub async fn account_mode(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<PerpetualAccountMode, GemServiceError> {
        match self.gateway.get_perpetual_account_mode(chain, address).await {
            Ok(mode) => {
                self.wallet_preferences.set_perpetual_account_mode(wallet_id, mode)?;
                Ok(mode)
            }
            Err(_) => self.wallet_preferences.get_perpetual_account_mode(wallet_id),
        }
    }

    pub fn markets_updated_at(&self) -> Result<Option<i64>, GemServiceError> {
        self.preferences.get_perpetual_markets_updated_at()
    }

    pub async fn sync_markets(&self, chain: Chain) -> Result<(), GemServiceError> {
        let currency = self.preferences.get_currency();
        let data = self.gateway.get_perpetuals_data(chain).await?;
        self.store.save_perpetuals(data).await?;
        if let Some(price) = rules::collateral_price(chain) {
            self.price.update_prices(vec![price], currency).await?;
        }
        self.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp()))
    }

    pub async fn sync_markets_if_stale(&self, chain: Chain) -> Result<bool, GemServiceError> {
        if !rules::is_markets_stale(self.markets_updated_at()?, Utc::now().timestamp()) {
            return Ok(false);
        }
        self.sync_markets(chain).await?;
        Ok(true)
    }

    pub async fn get_candlesticks(&self, chain: Chain, symbol: String, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        Ok(self.gateway.get_perpetual_candlesticks(chain, symbol, period.as_ref().to_string()).await?)
    }

    pub async fn get_portfolio(&self, chain: Chain, address: String) -> Result<PerpetualPortfolio, GemServiceError> {
        Ok(self.gateway.get_perpetual_portfolio(chain, address).await?)
    }

    pub async fn clear_markets(&self) -> Result<(), GemServiceError> {
        self.store.clear().await?;
        self.preferences.set_perpetual_markets_updated_at(None)
    }

    pub async fn set_pinned(&self, perpetual_id: String, pinned: bool) -> Result<(), GemServiceError> {
        self.store.set_pinned(vec![perpetual_id], pinned).await
    }

    pub async fn apply_socket_message(&self, wallet_id: WalletId, mode: PerpetualAccountMode, data: Vec<u8>) -> Result<GemPerpetualSocketUpdate, GemServiceError> {
        let message = parse_websocket_data(&data, mode).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        match message {
            HyperliquidSocketMessage::AccountState { balance, positions } => {
                let existing = self.store.get_positions(wallet_id.clone(), PerpetualProvider::Hypercore).await?;
                let diff = diff_clearinghouse_positions(positions, existing);
                self.store.update_positions(wallet_id.clone(), diff.positions, diff.delete_position_ids).await?;
                if let Some(balance) = balance {
                    self.update_balance(wallet_id, balance).await?;
                }
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::SpotState { balance } => {
                self.update_balance(wallet_id, balance).await?;
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::OpenOrders { orders } => {
                let existing = self.store.get_positions(wallet_id.clone(), PerpetualProvider::Hypercore).await?;
                let diff = diff_open_orders_positions(&orders, existing);
                self.store.update_positions(wallet_id, diff.positions, diff.delete_position_ids).await?;
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::Candle { candle } => Ok(GemPerpetualSocketUpdate::Candle { candle }),
            HyperliquidSocketMessage::MarketData { market } => {
                self.store.update_market(market).await?;
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::MarketPrices { prices } => {
                self.update_prices(prices).await?;
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::SubscriptionResponse { subscription_type } => Ok(GemPerpetualSocketUpdate::SubscriptionResponse { subscription_type }),
            HyperliquidSocketMessage::Error { message } => Ok(GemPerpetualSocketUpdate::Error { message }),
            HyperliquidSocketMessage::Unknown => Ok(GemPerpetualSocketUpdate::Unknown),
        }
    }

    pub async fn sync_positions(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<PerpetualAccountMode, GemServiceError> {
        let mode = self.account_mode(wallet_id.clone(), chain, address.clone()).await?;
        let summary = self.gateway.get_positions(chain, address).await?;
        let existing_ids = self.store.get_position_ids(wallet_id.clone(), provider(chain)?).await?;
        let delete_ids = rules::stale_position_ids(existing_ids, &summary.positions);
        self.store.update_positions(wallet_id.clone(), summary.positions, delete_ids).await?;
        self.update_balance(wallet_id, summary.balance).await?;
        Ok(mode)
    }
}

impl GemPerpetualService {
    pub async fn update_balance(&self, wallet_id: WalletId, balance: PerpetualBalance) -> Result<(), GemServiceError> {
        let update = rules::balance_update(&balance).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
        self.balance.update_balances(wallet_id, vec![update]).await
    }
    pub async fn update_prices(&self, prices: HashMap<String, f64>) -> Result<(), GemServiceError> {
        let now = Utc::now().timestamp();
        if !rules::prices_outdated(self.preferences.get_perpetual_prices_updated_at()?, now, PRICES_UPDATE_INTERVAL_SECONDS) {
            return Ok(());
        }
        self.store.update_prices(prices).await?;
        self.preferences.set_perpetual_prices_updated_at(Some(now))
    }
}

fn provider(chain: Chain) -> Result<PerpetualProvider, GemServiceError> {
    rules::provider(chain).ok_or_else(|| GemServiceError::Unsupported {
        msg: format!("perpetuals unsupported on {chain}"),
    })
}

#[uniffi::export]
pub fn perpetual_collateral_asset_id(chain: Chain) -> Option<AssetId> {
    rules::collateral_asset_id(chain)
}
