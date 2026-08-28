pub mod model;
pub mod rules;
pub mod store;

use crate::perpetual::Perpetual;
use model::{GemPerpetualCloseInput, GemPerpetualOrderAction, GemPerpetualOrderInput};

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::Utc;
use gem_hypercore::models::websocket::HyperliquidSocketMessage;
use gem_hypercore::provider::websocket_mapper::{diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data};
use num_bigint::BigInt;
use primitives::perpetual::PerpetualBalance;
use primitives::portfolio::PerpetualPortfolio;
use primitives::{AssetId, Chain, ChartPeriod, PerpetualAccountMode, PerpetualConfirmData, PerpetualProvider, PerpetualReduceData, PerpetualType, Wallet, WalletId};
use std::collections::HashMap;
use std::str::FromStr;

use crate::config::perpetual_config::PRICES_UPDATE_INTERVAL_SECONDS;
use crate::services::preferences::GemPreferencesService;

pub use model::GemPerpetualSocketUpdate;
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

    pub async fn sync_markets_if_stale(&self, chain: Chain) -> Result<bool, GemServiceError> {
        if !rules::is_markets_stale(self.markets_updated_at()?, Utc::now().timestamp()) {
            return Ok(false);
        }
        self.sync_markets(chain).await?;
        Ok(true)
    }

    pub async fn sync_enablement(&self, wallet: Option<Wallet>) -> Result<bool, GemServiceError> {
        if !self.preferences.is_perpetual_enabled() {
            self.clear_markets().await?;
            return Ok(false);
        }
        self.sync_markets_if_stale(Chain::HyperCore).await?;
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

#[uniffi::export]
pub fn perpetual_funding_apr(funding: f64) -> f64 {
    rules::funding_apr(funding)
}

#[uniffi::export]
pub fn perpetual_order(input: GemPerpetualOrderInput) -> Result<PerpetualType, GemServiceError> {
    let usdc_amount = BigInt::from_str(&input.usdc_amount).map_err(|error| GemServiceError::InvalidInput { msg: error.to_string() })?;
    let usd_amount = usdc_amount.to_string().parse::<f64>().unwrap_or_default() / 10f64.powi(input.usdc_decimals);
    let slippage = rules::slippage_percent(input.slippage);
    let (size, fiat_value, margin_amount) = rules::order_amounts(usd_amount, input.leverage, input.price);
    let price = rules::slippage_price(input.price, input.direction.clone(), rules::opens_position(&input.action), slippage);
    let formatter = Perpetual::new(input.provider);

    let data = PerpetualConfirmData {
        direction: input.direction,
        margin_type: input.margin_type,
        base_asset: input.base_asset,
        asset_index: input.asset_index,
        price: formatter.format_price(price, input.asset.decimals),
        fiat_value,
        size: formatter.format_size(size, input.asset.decimals),
        slippage,
        leverage: input.leverage,
        pnl: None,
        entry_price: None,
        market_price: input.price,
        margin_amount,
        take_profit: input.take_profit,
        stop_loss: input.stop_loss,
    };

    Ok(match input.action {
        GemPerpetualOrderAction::Open => PerpetualType::Open(data),
        GemPerpetualOrderAction::Increase => PerpetualType::Increase(data),
        GemPerpetualOrderAction::Reduce { position_direction } => PerpetualType::Reduce(PerpetualReduceData { data, position_direction }),
    })
}

#[uniffi::export]
pub fn perpetual_close_order(input: GemPerpetualCloseInput) -> PerpetualConfirmData {
    let slippage = rules::slippage_percent(input.slippage);
    let price = rules::slippage_price(input.market_price, input.direction.clone(), false, slippage);
    let size = input.size.abs();
    let formatter = Perpetual::new(input.provider);

    PerpetualConfirmData {
        direction: input.direction,
        margin_type: input.margin_type,
        base_asset: input.base_asset,
        asset_index: input.asset_index,
        price: formatter.format_price(price, input.asset.decimals),
        fiat_value: size * price,
        size: formatter.format_size(size, input.asset.decimals),
        slippage,
        leverage: input.leverage,
        pnl: Some(input.pnl),
        entry_price: Some(input.entry_price),
        market_price: input.market_price,
        margin_amount: input.margin_amount,
        take_profit: None,
        stop_loss: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, PerpetualDirection, PerpetualMarginType};

    fn order_input(action: GemPerpetualOrderAction) -> GemPerpetualOrderInput {
        GemPerpetualOrderInput {
            action,
            direction: PerpetualDirection::Long,
            margin_type: PerpetualMarginType::Cross,
            base_asset: Asset::mock(),
            asset: Asset::mock(),
            asset_index: 1,
            provider: PerpetualProvider::Hypercore,
            price: 100.0,
            usdc_amount: "50000000".to_string(),
            usdc_decimals: 6,
            leverage: 4,
            slippage: None,
            take_profit: None,
            stop_loss: None,
        }
    }

    #[test]
    fn test_perpetual_order_keeps_the_position_action_and_prices_in_the_slippage() {
        let open = perpetual_order(order_input(GemPerpetualOrderAction::Open)).unwrap();
        let increase = perpetual_order(order_input(GemPerpetualOrderAction::Increase)).unwrap();
        let reduce = perpetual_order(order_input(GemPerpetualOrderAction::Reduce {
            position_direction: PerpetualDirection::Short,
        }))
        .unwrap();

        let PerpetualType::Open(data) = open else { panic!("expected an open order") };
        assert_eq!(data.slippage, 2.0);
        assert_eq!(data.market_price, 100.0);
        assert_eq!(data.fiat_value, 200.0);
        assert_eq!(data.margin_amount, 50.0);
        assert!(matches!(increase, PerpetualType::Increase(_)));
        let PerpetualType::Reduce(reduce) = reduce else { panic!("expected a reduce order") };
        assert_eq!(reduce.position_direction, PerpetualDirection::Short);
    }

    #[test]
    fn test_perpetual_close_order_carries_the_position_result() {
        let data = perpetual_close_order(GemPerpetualCloseInput {
            asset_index: 1,
            direction: PerpetualDirection::Long,
            margin_type: PerpetualMarginType::Cross,
            base_asset: Asset::mock(),
            asset: Asset::mock(),
            provider: PerpetualProvider::Hypercore,
            market_price: 100.0,
            size: -2.0,
            leverage: 4,
            pnl: 12.5,
            entry_price: 90.0,
            margin_amount: 50.0,
            slippage: None,
        });

        assert_eq!(data.pnl, Some(12.5));
        assert_eq!(data.entry_price, Some(90.0));
        assert_eq!(data.fiat_value, 196.0);
    }
}
