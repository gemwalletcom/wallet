pub mod model;
pub mod rules;
pub mod store;
pub mod stream;

use crate::services::error::GemServiceError;
use model::GemPerpetualConnection;
use std::sync::Arc;

use chrono::Utc;
use gem_hypercore::models::websocket::HyperliquidSocketMessage;
use gem_hypercore::provider::websocket_mapper::{diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data};
use primitives::perpetual::PerpetualBalance;
use primitives::portfolio::PerpetualPortfolio;
use primitives::{AssetId, Chain, ChartPeriod, PerpetualAccountMode, PerpetualModifyConfirmData, PerpetualProvider, Wallet, WalletId};
use std::collections::HashMap;

use crate::config::perpetual_config::PRICES_UPDATE_INTERVAL_SECONDS;
use crate::services::preferences::GemPreferencesService;

pub use model::{GemAutocloseSummary, GemMarketsRefreshTrigger, GemPerpetualSocketUpdate};
pub use store::GemPerpetualStore;

use crate::gateway::GemGateway;
use crate::models::perpetual::GemChartCandleStick;
use crate::services::assets::GemAssetStore;
use crate::services::balance::GemBalanceService;
use crate::services::price::GemPriceService;
use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemPerpetualService {
    gateway: Arc<GemGateway>,
    price: Arc<GemPriceService>,
    store: Arc<dyn GemPerpetualStore>,
    asset_store: Arc<dyn GemAssetStore>,
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
        asset_store: Arc<dyn GemAssetStore>,
        preferences: Arc<GemPreferencesService>,
        balance: Arc<GemBalanceService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            gateway,
            price,
            store,
            asset_store,
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

    pub fn autoclose_summary(&self, data: PerpetualModifyConfirmData) -> Option<GemAutocloseSummary> {
        rules::autoclose_summary(&data)
    }

    pub fn markets_updated_at(&self) -> Result<Option<i64>, GemServiceError> {
        self.preferences.get_perpetual_markets_updated_at()
    }

    pub async fn sync_markets(&self, chain: Chain) -> Result<(), GemServiceError> {
        let currency = self.preferences.get_currency();
        let data = self.gateway.get_perpetuals_data(chain).await?;
        self.asset_store.save_assets(rules::perpetual_asset_basics(&data)).await?;
        self.store.save_perpetuals(data).await?;
        if let Some(price) = rules::collateral_price(chain) {
            self.price.update_prices(vec![price], currency).await?;
        }
        self.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp()))
    }

    pub async fn sync_markets_if_needed(&self, chain: Chain, trigger: GemMarketsRefreshTrigger) -> Result<bool, GemServiceError> {
        if !trigger.should_sync_markets(self.markets_updated_at()?, Utc::now().timestamp()) {
            return Ok(false);
        }
        self.sync_markets(chain).await?;
        Ok(true)
    }

    pub async fn sync_enablement(&self, wallet: Option<Wallet>, trigger: GemMarketsRefreshTrigger) -> Result<bool, GemServiceError> {
        if !self.preferences.is_perpetual_enabled() {
            self.clear_markets().await?;
            return Ok(false);
        }
        self.sync_markets_if_needed(Chain::HyperCore, trigger).await?;
        Ok(self.should_connect_perpetuals(wallet))
    }

    pub fn should_connect_perpetuals(&self, wallet: Option<Wallet>) -> bool {
        wallet
            .as_ref()
            .is_some_and(|wallet| rules::show_perpetuals(self.preferences.is_perpetual_enabled(), wallet))
    }

    pub async fn get_candlesticks(&self, chain: Chain, symbol: String, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        Ok(self.gateway.get_perpetual_candlesticks(chain, symbol, period.as_ref().to_string()).await?)
    }

    pub fn candle_interval(&self, period: ChartPeriod) -> String {
        rules::candle_interval(&period).to_string()
    }

    pub fn merge_candle(&self, candles: Vec<GemChartCandleStick>, candle: GemChartCandleStick) -> Vec<GemChartCandleStick> {
        rules::merge_candle(candles, candle)
    }

    pub async fn get_portfolio(&self, chain: Chain, address: String) -> Result<PerpetualPortfolio, GemServiceError> {
        Ok(self.gateway.get_perpetual_portfolio(chain, address).await?)
    }

    pub async fn clear_markets(&self) -> Result<(), GemServiceError> {
        self.store.delete_perpetuals().await?;
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

    pub async fn connection(&self, wallet: Wallet) -> Result<Option<GemPerpetualConnection>, GemServiceError> {
        let Some(account) = crate::services::stream::rules::hyperliquid_account(&wallet.accounts) else {
            return Ok(None);
        };
        let chain = account.chain;
        let address = account.address.clone();
        let mode = match self.sync_positions(wallet.id.clone(), chain, address.clone()).await {
            Ok(mode) => mode,
            Err(_) => self.account_mode(wallet.id, chain, address.clone()).await?,
        };
        Ok(Some(GemPerpetualConnection { address, mode }))
    }

    pub fn collateral_asset_id(&self, chain: Chain) -> Option<AssetId> {
        rules::collateral_asset_id(chain)
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
