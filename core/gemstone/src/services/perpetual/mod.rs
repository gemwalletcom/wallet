pub mod autoclose;
pub mod candles;
pub mod details;
pub mod model;
pub mod rules;
pub mod store;
pub mod stream;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use crate::services::failures::record;
use model::{GemPerpetualConnection, GemPerpetualRefreshFailure, GemPerpetualRefreshStep};
use std::sync::Arc;

use chrono::Utc;
use gem_hypercore::models::websocket::HyperliquidSocketMessage;
use gem_hypercore::provider::websocket_mapper::{diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data};
use primitives::perpetual::{PerpetualAccountPositions, PerpetualBalance, PerpetualData};
use primitives::portfolio::PerpetualPortfolio;
use primitives::{Asset, AssetId, Chain, ChartPeriod, PerpetualAccountMode, PerpetualProvider, Wallet, WalletId, WalletType};
use std::collections::HashMap;

use crate::config::perpetual_config::PRICES_UPDATE_INTERVAL_SECONDS;
use crate::services::preferences::GemPreferencesService;

pub use autoclose::{GemAutocloseEstimate, GemAutocloseField, GemAutocloseModify};
pub use candles::{GemCandleRequest, GemCandleResult, GemCandleSession, GemCandleViewState};
pub use details::GemPerpetualDetailsService;
pub use model::{
    GemMarketsRefreshTrigger, GemPerpetualButton, GemPerpetualDetails, GemPerpetualEnablementTrigger, GemPerpetualMarketCounts, GemPerpetualPositionAction, GemPerpetualPositionDetailRow, GemPerpetualPositionKind, GemPerpetualSection,
    GemPerpetualSocketUpdate, GemPerpetualTransferData,
};
pub use store::GemPerpetualStore;

use crate::gateway::GemGateway;
use crate::models::perpetual::GemChartCandleStick;
use crate::services::assets::{GemAssetAction, GemAssetsService};
use crate::services::balance::GemBalanceService;
use crate::services::price::GemPriceService;
use crate::services::stream::rules::hyperliquid_account;
use crate::services::transfer::GemRecentActivityService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemPerpetualService {
    gateway: Arc<GemGateway>,
    price: Arc<GemPriceService>,
    store: Arc<dyn GemPerpetualStore>,
    assets: Arc<GemAssetsService>,
    preferences: Arc<GemPreferencesService>,
    balance: Arc<GemBalanceService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
    session: Arc<GemWalletSessionService>,
    recent_activity: Arc<GemRecentActivityService>,
}

#[uniffi::export]
impl GemPerpetualService {
    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        price: Arc<GemPriceService>,
        store: Arc<dyn GemPerpetualStore>,
        assets: Arc<GemAssetsService>,
        preferences: Arc<GemPreferencesService>,
        balance: Arc<GemBalanceService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
        session: Arc<GemWalletSessionService>,
        recent_activity: Arc<GemRecentActivityService>,
    ) -> Self {
        Self {
            gateway,
            price,
            store,
            assets,
            preferences,
            balance,
            wallet_preferences,
            session,
            recent_activity,
        }
    }

    pub async fn add_recent(&self, action: GemAssetAction, asset: Asset) -> Result<(), GemServiceError> {
        self.recent_activity.add_recent(action, asset).await
    }

    pub async fn refresh(&self, trigger: GemMarketsRefreshTrigger) -> Vec<GemPerpetualRefreshFailure> {
        let (positions, markets) = futures::join!(self.sync_current_positions(), async { self.sync_markets_if_needed(Chain::HyperCore, trigger).await.map(|_| ()) });
        let mut failures = Vec::new();
        record(&mut failures, GemPerpetualRefreshStep::Positions, async { positions }).await;
        record(&mut failures, GemPerpetualRefreshStep::Markets, async { markets }).await;
        failures
    }

    pub async fn sync_enablement(&self, wallet: Option<Wallet>, trigger: GemPerpetualEnablementTrigger) -> Result<bool, GemServiceError> {
        if !self.preferences.is_perpetual_enabled() {
            self.clear_markets().await?;
            return Ok(false);
        }
        if trigger != GemPerpetualEnablementTrigger::Foreground {
            self.sync_markets_if_needed(Chain::HyperCore, GemMarketsRefreshTrigger::Scheduled).await?;
        }
        Ok(self.should_connect_perpetuals(wallet))
    }

    pub fn show_perpetuals(&self, wallet_type: WalletType, chains: Vec<Chain>) -> bool {
        self.preferences.show_perpetuals(wallet_type, chains)
    }

    pub async fn set_pinned(&self, perpetual_id: String, pinned: bool) -> Result<(), GemServiceError> {
        self.store.set_pinned(vec![perpetual_id], pinned).await
    }

    pub async fn connection(&self, wallet: Wallet) -> Result<Option<GemPerpetualConnection>, GemServiceError> {
        let Some(account) = hyperliquid_account(&wallet.accounts) else {
            return Ok(None);
        };
        let chain = Chain::HyperCore;
        let address = account.address.clone();
        let mode = match self.sync_positions(wallet.id.clone(), chain, address.clone()).await {
            Ok(mode) => mode,
            Err(_) => self.account_mode(wallet.id, chain, address.clone()).await?,
        };
        Ok(Some(GemPerpetualConnection { address, mode }))
    }
}

impl GemPerpetualService {
    fn should_connect_perpetuals(&self, wallet: Option<Wallet>) -> bool {
        wallet.is_some_and(|wallet| self.preferences.show_perpetuals(wallet.wallet_type, wallet.chains()))
    }

    pub async fn sync_markets_if_needed(&self, chain: Chain, trigger: GemMarketsRefreshTrigger) -> Result<bool, GemServiceError> {
        if !trigger.should_sync_markets(self.markets_updated_at()?, Utc::now().timestamp()) {
            return Ok(false);
        }
        self.sync_markets(chain).await?;
        Ok(true)
    }

    pub async fn sync_current_positions(&self) -> Result<(), GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        let Some(account) = hyperliquid_account(&wallet.accounts) else {
            return Ok(());
        };
        self.sync_positions(wallet.id, Chain::HyperCore, account.address.clone()).await.map(|_| ())
    }

    pub async fn sync_markets(&self, chain: Chain) -> Result<(), GemServiceError> {
        let data = self.gateway.get_perpetuals_data(chain).await?;
        self.save_markets(data).await?;
        if let Some(price) = rules::collateral_price(chain) {
            self.price.update_prices(vec![price]).await?;
        }
        self.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp()))
    }

    pub async fn save_markets(&self, data: Vec<PerpetualData>) -> Result<(), GemServiceError> {
        self.assets.save_assets(rules::perpetual_asset_basics(&data)).await?;
        let stored = self.store.get_perpetuals(data.iter().map(|data| data.perpetual.name.clone()).collect()).await?;
        let changed = rules::changed_perpetuals(data, &stored);
        if changed.is_empty() {
            return Ok(());
        }
        self.store.save_perpetuals(changed).await
    }

    pub async fn get_portfolio(&self, chain: Chain, address: String) -> Result<PerpetualPortfolio, GemServiceError> {
        Ok(self.gateway.get_perpetual_portfolio(chain, address).await?)
    }

    pub async fn on_socket_message(&self, wallet_id: WalletId, mode: PerpetualAccountMode, data: Vec<u8>) -> Result<GemPerpetualSocketUpdate, GemServiceError> {
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
                let stored = self.store.get_perpetuals(vec![market.coin.clone()]).await?;
                if rules::market_changed(&market, &stored) {
                    self.store.update_market(market).await?;
                }
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
}

impl GemPerpetualService {
    pub async fn sync_positions(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<PerpetualAccountMode, GemServiceError> {
        let PerpetualAccountPositions { mode, summary } = self.gateway.get_positions(chain, address).await?;
        self.wallet_preferences.set_perpetual_account_mode(wallet_id.clone(), mode)?;
        let existing_ids = self.store.get_position_ids(wallet_id.clone(), provider(chain)?).await?;
        let delete_ids = rules::stale_position_ids(existing_ids, &summary.positions);
        self.store.update_positions(wallet_id.clone(), summary.positions, delete_ids).await?;
        self.update_balance(wallet_id, summary.balance).await?;
        Ok(mode)
    }

    async fn account_mode(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<PerpetualAccountMode, GemServiceError> {
        match self.gateway.get_perpetual_account_mode(chain, address).await {
            Ok(mode) => {
                self.wallet_preferences.set_perpetual_account_mode(wallet_id, mode)?;
                Ok(mode)
            }
            Err(_) => self.wallet_preferences.get_perpetual_account_mode(wallet_id),
        }
    }
}

impl GemPerpetualService {
    pub fn markets_updated_at(&self) -> Result<Option<i64>, GemServiceError> {
        self.preferences.get_perpetual_markets_updated_at()
    }

    pub async fn get_candlesticks(&self, chain: Chain, symbol: String, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        Ok(self.gateway.get_perpetual_candlesticks(chain, symbol, period.as_ref().to_string()).await?)
    }

    pub async fn clear_markets(&self) -> Result<(), GemServiceError> {
        self.store.clear_perpetuals(rules::collateral_asset_ids()).await?;
        self.preferences.set_perpetual_markets_updated_at(None)
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
        let stored = self.store.get_perpetuals(prices.keys().cloned().collect()).await?;
        let changed = rules::changed_perpetual_prices(prices, &stored);
        if !changed.is_empty() {
            self.store.update_prices(changed).await?;
        }
        self.preferences.set_perpetual_prices_updated_at(Some(now))
    }
}

fn provider(chain: Chain) -> Result<PerpetualProvider, GemServiceError> {
    rules::provider(chain).ok_or_else(|| GemServiceError::Unsupported {
        msg: format!("perpetuals unsupported on {chain}"),
    })
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::Account;

    use super::testkit::PerpetualTestkit;
    use super::*;

    const ALL_MIDS: &str = r#"{"channel":"allMids","data":{"mids":{"BTC":"104633.0","ETH":"3321.1"}}}"#;
    const OPEN_ORDERS: &str = r#"{"channel":"openOrders","data":{"user":"0xc64c","orders":[{"coin":"BTC","oid":1,"triggerPx":"110000.0","limitPx":"110000.0","isPositionTpsl":true,"orderType":"Take Profit Market"}]}}"#;
    const CANDLE: &str = r#"{"channel":"candle","data":{"t":1700000000000,"T":1700000059999,"s":"BTC","i":"1m","o":"1.0","c":"2.0","h":"3.0","l":"0.5","v":"10.0","n":3}}"#;
    const SUBSCRIPTION: &str = r#"{"channel":"subscriptionResponse","data":{"method":"subscribe","subscription":{"type":"allMids"}}}"#;

    fn message(testkit: &PerpetualTestkit, payload: &str) -> GemPerpetualSocketUpdate {
        block_on(testkit.service.on_socket_message(testkit.wallet_id.clone(), PerpetualAccountMode::Standard, payload.as_bytes().to_vec())).unwrap()
    }

    #[test]
    fn test_a_price_message_writes_once_and_then_waits_out_the_interval() {
        let testkit = PerpetualTestkit::new();

        assert_eq!(message(&testkit, ALL_MIDS), GemPerpetualSocketUpdate::Applied);
        assert_eq!(message(&testkit, ALL_MIDS), GemPerpetualSocketUpdate::Applied);

        let writes = testkit.store.price_writes.lock().unwrap();
        assert_eq!(writes.len(), 1, "a second message inside the interval is dropped, not written again");
        assert_eq!(writes[0].get("BTC"), Some(&104_633.0));
        assert!(testkit.preferences.get_perpetual_prices_updated_at().unwrap().is_some());
    }

    #[test]
    fn test_an_unchanged_market_tick_writes_nothing() {
        let testkit = PerpetualTestkit::new();
        let tick = include_str!("../../../../crates/gem_hypercore/testdata/ws_active_asset_ctx.json");

        message(&testkit, tick);
        let written = testkit.store.markets.lock().unwrap()[0].clone();
        *testkit.store.stored.lock().unwrap() = vec![primitives::perpetual::Perpetual {
            name: written.coin.clone(),
            price: written.price,
            price_percent_change_24h: written.price_percent_change_24h,
            open_interest: written.open_interest,
            volume_24h: written.volume_24h,
            funding: written.funding,
            ..primitives::perpetual::Perpetual::mock()
        }];
        message(&testkit, tick);

        assert_eq!(testkit.store.markets.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_a_price_tick_writes_only_the_prices_that_moved() {
        let testkit = PerpetualTestkit::new();
        *testkit.store.stored.lock().unwrap() = vec![primitives::perpetual::Perpetual {
            name: "BTC".to_string(),
            price: 104_633.0,
            ..primitives::perpetual::Perpetual::mock()
        }];

        message(&testkit, ALL_MIDS);

        let writes = testkit.store.price_writes.lock().unwrap();
        assert_eq!(writes[0].keys().collect::<Vec<_>>(), vec!["ETH"]);
    }

    #[test]
    fn test_an_unchanged_market_list_writes_no_perpetuals() {
        let testkit = PerpetualTestkit::new();
        let data = primitives::perpetual::PerpetualData {
            perpetual: primitives::perpetual::Perpetual::mock(),
            asset: primitives::Asset::from_chain(Chain::HyperCore),
            metadata: primitives::perpetual::PerpetualMetadata { is_pinned: false },
        };
        *testkit.store.stored.lock().unwrap() = vec![data.perpetual.clone()];

        block_on(testkit.service.save_markets(vec![data])).unwrap();

        assert!(testkit.store.perpetual_writes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_an_orders_message_diffs_against_the_positions_the_store_already_has() {
        let testkit = PerpetualTestkit::new();

        assert_eq!(message(&testkit, OPEN_ORDERS), GemPerpetualSocketUpdate::Applied);

        assert_eq!(testkit.store.position_writes.lock().unwrap().len(), 1);
        assert!(testkit.balances.balance_writes.lock().unwrap().is_empty(), "an orders message is not a balance");
    }

    #[test]
    fn test_a_candle_is_handed_back_without_touching_the_wallet() {
        let testkit = PerpetualTestkit::new();

        let update = message(&testkit, CANDLE);

        assert!(matches!(update, GemPerpetualSocketUpdate::Candle { .. }));
        assert!(testkit.store.position_writes.lock().unwrap().is_empty());
        assert!(testkit.store.price_writes.lock().unwrap().is_empty());
        assert!(testkit.balances.balance_writes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_a_subscription_reply_and_an_unreadable_channel_are_reported_not_applied() {
        let testkit = PerpetualTestkit::new();

        assert!(matches!(message(&testkit, SUBSCRIPTION), GemPerpetualSocketUpdate::SubscriptionResponse { .. }));
        assert_eq!(message(&testkit, r#"{"channel":"somethingElse"}"#), GemPerpetualSocketUpdate::Unknown);
        assert!(testkit.store.position_writes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_a_new_channel_that_carries_a_payload_is_reported_as_an_error() {
        let testkit = PerpetualTestkit::new();

        let update = block_on(
            testkit
                .service
                .on_socket_message(testkit.wallet_id.clone(), PerpetualAccountMode::Standard, br#"{"channel":"somethingElse","data":{}}"#.to_vec()),
        );

        assert!(
            matches!(update, Err(GemServiceError::Core { .. })),
            "only a bare unknown channel reads as Unknown; one carrying a payload is reported so a new Hyperliquid channel is noticed"
        );
    }

    #[test]
    fn test_a_payload_that_is_not_a_socket_message_is_an_error() {
        let testkit = PerpetualTestkit::new();

        let error = block_on(testkit.service.on_socket_message(testkit.wallet_id.clone(), PerpetualAccountMode::Standard, b"not json".to_vec()));

        assert!(matches!(error, Err(GemServiceError::Core { .. })), "{error:?}");
    }

    #[test]
    fn test_disabled_perpetuals_clear_the_markets_without_asking_the_gateway() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_enabled(false).unwrap();

            assert!(!testkit.service.sync_enablement(None, GemPerpetualEnablementTrigger::PreferenceChanged).await.unwrap());

            assert_eq!(*testkit.store.deleted.lock().unwrap(), 1);
            assert_eq!(
                testkit.store.cleared_collateral.lock().unwrap().clone(),
                vec![vec![primitives::known_assets::HYPERCORE_PERPETUAL_USDC.id.clone()]],
                "the store is told which collateral the clear takes with the markets"
            );
            assert!(testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_returning_to_the_foreground_decides_the_connection_without_refreshing_markets() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_enabled(true).unwrap();

            assert!(!testkit.service.sync_enablement(None, GemPerpetualEnablementTrigger::Foreground).await.unwrap(), "no wallet, nothing to connect");
            assert!(testkit.provider.requested_paths().is_empty(), "the foreground leaves the markets to the other triggers");
        })
    }

    #[test]
    fn test_a_scheduled_refresh_skips_markets_that_were_just_synced() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp())).unwrap();

            assert!(!testkit.service.sync_markets_if_needed(Chain::HyperCore, GemMarketsRefreshTrigger::Scheduled).await.unwrap());

            assert!(testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_a_user_requested_refresh_always_asks_for_the_markets() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp())).unwrap();

            assert!(testkit.service.sync_markets_if_needed(Chain::HyperCore, GemMarketsRefreshTrigger::UserRequested).await.is_err());

            assert!(!testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_refresh_reports_both_steps_when_both_fail() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));
            *testkit.wallets.wallets.lock().unwrap() = vec![wallet.clone()];
            testkit.service.session.set_current_wallet_id(Some(wallet.id.clone())).unwrap();

            let failures = testkit.service.refresh(GemMarketsRefreshTrigger::UserRequested).await;

            let steps: Vec<GemPerpetualRefreshStep> = failures.iter().map(|failure| failure.step).collect();
            assert!(steps.contains(&GemPerpetualRefreshStep::Markets), "{failures:?}");
            assert!(steps.contains(&GemPerpetualRefreshStep::Positions), "{failures:?}");
        })
    }

    #[test]
    fn test_a_wallet_without_a_hypercore_account_syncs_no_positions() {
        block_on(async {
            let testkit = PerpetualTestkit::new();

            testkit.service.sync_current_positions().await.unwrap();

            assert!(testkit.provider.requested_paths().is_empty());
            assert!(testkit.store.position_writes.lock().unwrap().is_empty());
        })
    }

    #[test]
    fn test_connection_uses_hypercore_for_evm_accounts() {
        block_on(async {
            for chains in [[Chain::Arbitrum, Chain::HyperCore], [Chain::Hyperliquid, Chain::HyperCore], [Chain::HyperCore, Chain::Arbitrum]] {
                let testkit = PerpetualTestkit::with_unified_balance().await;
                let wallet = Wallet::mock_with_accounts(Account::mock_chains(&chains, "0xc64c"));

                let connection = testkit.service.connection(wallet.clone()).await.unwrap();

                assert_eq!(
                    connection,
                    Some(GemPerpetualConnection {
                        address: "0xc64c".to_string(),
                        mode: PerpetualAccountMode::Unified
                    })
                );
                let stored = testkit.balances.balances.lock().unwrap();
                assert_eq!(stored[&wallet.id][0].available.to_string(), "12093224");
                assert_eq!(stored[&wallet.id][0].withdrawable.to_string(), "12093224");
            }
        });
    }

    #[test]
    fn test_refresh_uses_hypercore_for_evm_accounts() {
        block_on(async {
            for chains in [[Chain::Arbitrum, Chain::HyperCore], [Chain::Hyperliquid, Chain::HyperCore], [Chain::HyperCore, Chain::Arbitrum]] {
                let testkit = PerpetualTestkit::with_unified_balance().await;
                let wallet = Wallet::mock_with_accounts(Account::mock_chains(&chains, "0xc64c"));
                *testkit.wallets.wallets.lock().unwrap() = vec![wallet.clone()];
                testkit.service.session.set_current_wallet_id(Some(wallet.id.clone())).unwrap();
                testkit.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp())).unwrap();

                assert_eq!(testkit.service.refresh(GemMarketsRefreshTrigger::Scheduled).await, vec![]);

                let stored = testkit.balances.balances.lock().unwrap();
                assert_eq!(stored[&wallet.id][0].available.to_string(), "12093224");
                assert_eq!(stored[&wallet.id][0].withdrawable.to_string(), "12093224");
                assert_eq!(testkit.wallet_preferences.get_perpetual_account_mode(wallet.id).unwrap(), PerpetualAccountMode::Unified);
            }
        });
    }

    #[test]
    fn test_connection_and_refresh_each_ask_for_the_account_mode_once() {
        block_on(async {
            let testkit = PerpetualTestkit::with_unified_balance().await;
            let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));
            *testkit.wallets.wallets.lock().unwrap() = vec![wallet.clone()];
            testkit.service.session.set_current_wallet_id(Some(wallet.id.clone())).unwrap();

            testkit.service.connection(wallet).await.unwrap();
            testkit.service.sync_current_positions().await.unwrap();

            let mode_requests = testkit.provider.requested_types().iter().filter(|kind| *kind == "userAbstraction").count();
            assert_eq!(mode_requests, 2);
        })
    }

    #[test]
    fn test_perpetuals_connect_only_for_a_wallet_that_can_hold_them() {
        let testkit = PerpetualTestkit::new();
        testkit.preferences.set_perpetual_enabled(true).unwrap();
        let hypercore = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));

        assert!(!testkit.service.should_connect_perpetuals(None));
        assert!(testkit.service.should_connect_perpetuals(Some(hypercore)));
    }

    #[test]
    fn test_an_unreachable_account_mode_falls_back_to_the_stored_one() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.wallet_preferences.set_perpetual_account_mode(testkit.wallet_id.clone(), PerpetualAccountMode::Unified).unwrap();

            let mode = testkit.service.account_mode(testkit.wallet_id.clone(), Chain::HyperCore, "0xc64c".to_string()).await.unwrap();

            assert_eq!(mode, PerpetualAccountMode::Unified);
        })
    }

    #[test]
    fn test_an_account_with_no_stored_mode_reads_as_standard() {
        block_on(async {
            let testkit = PerpetualTestkit::new();

            let mode = testkit.service.account_mode(testkit.wallet_id.clone(), Chain::HyperCore, "0xc64c".to_string()).await.unwrap();

            assert_eq!(mode, PerpetualAccountMode::Standard);
        })
    }
}
