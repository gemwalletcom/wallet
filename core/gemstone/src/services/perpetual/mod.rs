pub mod autoclose;
pub mod candles;
pub mod chart;
pub mod details;
pub mod model;
pub mod rules;
pub mod store;
pub mod stream;
#[cfg(test)]
pub(crate) mod testkit;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use futures::lock::Mutex;
use gem_hypercore::config::HypercoreConfig;
use gem_hypercore::models::perp_dex::PerpDex;
use gem_hypercore::models::websocket::HyperliquidSocketMessage;
use gem_hypercore::provider::perpetual_mapper::total_dexes_balance;
use gem_hypercore::provider::websocket_mapper::{diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data};
use primitives::perpetual::{PerpetualAccountPositions, PerpetualBalance, PerpetualData};
use primitives::portfolio::PerpetualPortfolio;
use primitives::{Asset, AssetId, Chain, ChartPeriod, Feature, PerpetualAccountMode, PerpetualId, PerpetualPosition, PerpetualProvider, RecentActivityType, Wallet, WalletId};

use crate::config::perpetual_config::PRICES_UPDATE_INTERVAL_SECONDS;
use crate::gateway::GemGateway;
use crate::models::custom_types::GemBigUint;
use crate::models::perpetual::GemChartCandleStick;
use crate::perpetual::GemPerpetual;
use crate::services::assets::{GemAssetAction, GemAssetsService};
use crate::services::balance::GemBalanceService;
use crate::services::clock::is_outdated;
use crate::services::collections::stale;
use crate::services::config::GemConfigService;
use crate::services::error::GemServiceError;
use crate::services::failures::record;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
use crate::services::stream::rules::hyperliquid_account;
use crate::services::toast::GemToast;
use crate::services::transfer::{GemRecentActivityScope, GemRecentActivityService};
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;
use model::{GemPerpetualRefreshFailure, GemPerpetualRefreshStep};

pub use autoclose::{GemAutocloseEstimate, GemAutocloseField, GemAutocloseModify};
pub use candles::{GemCandleRequest, GemCandleResult, GemCandleSession, GemCandleViewState};
pub use details::GemPerpetualDetailsService;
pub use model::{
    GemMarketsRefreshTrigger, GemPerpetualButton, GemPerpetualDepositTarget, GemPerpetualDetails, GemPerpetualMarketCounts, GemPerpetualMarketView, GemPerpetualPositionAction, GemPerpetualPositionDetailRow, GemPerpetualPositionKind,
    GemPerpetualSection, GemPerpetualSocketUpdate, GemPerpetualTransferData,
};
pub use store::GemPerpetualStore;

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
    config: Arc<GemConfigService>,
    writes: Mutex<()>,
    markets_sync: Mutex<()>,
    pub(crate) perp_dexes: Vec<PerpDex>,
    dex_balances: std::sync::Mutex<HashMap<(WalletId, PerpDex), PerpetualBalance>>,
    pending_prices: std::sync::Mutex<HashMap<String, f64>>,
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
        config: Arc<GemConfigService>,
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
            config,
            writes: Mutex::new(()),
            markets_sync: Mutex::new(()),
            perp_dexes: HypercoreConfig::default().perp_dexes,
            dex_balances: std::sync::Mutex::new(HashMap::new()),
            pending_prices: std::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn is_available(&self) -> bool {
        self.config.is_feature_enabled(Feature::Perpetuals)
    }

    pub async fn add_recent(&self, action: GemAssetAction, asset: Asset) -> Result<(), GemServiceError> {
        self.recent_activity.add_recent(action, asset).await
    }

    pub async fn refresh(&self, trigger: GemMarketsRefreshTrigger) -> Vec<GemPerpetualRefreshFailure> {
        let account = async { (self.sync_current_positions().await, self.update_deposit_balances().await) };
        let ((positions, deposit_balances), markets) = futures::join!(account, async { self.sync_markets_if_needed(Chain::HyperCore, trigger).await.map(|_| ()) });
        let mut failures = Vec::new();
        record(&mut failures, GemPerpetualRefreshStep::Positions, async { positions }).await;
        record(&mut failures, GemPerpetualRefreshStep::Markets, async { markets }).await;
        record(&mut failures, GemPerpetualRefreshStep::DepositBalances, async { deposit_balances }).await;
        failures
    }

    pub async fn deposit_target(&self) -> Result<GemPerpetualDepositTarget, GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        let assets = self.deposit_assets(wallet.id.clone());
        let balances = self.balance.balances(wallet.id, assets.iter().map(|asset| asset.id.clone()).collect()).await?;
        let funded: Vec<AssetId> = balances.into_iter().filter(|balance| balance.available > GemBigUint::ZERO).map(|balance| balance.asset_id).collect();
        Ok(rules::deposit_target(assets, &funded))
    }

    pub async fn sync_enablement(&self, wallet: Option<Wallet>) -> Result<bool, GemServiceError> {
        if !self.preferences.is_perpetual_enabled() {
            self.clear_markets().await?;
            return Ok(false);
        }
        let _sync = self.markets_sync.lock().await;
        if self.preferences.get_perpetual_markets_updated_at().is_none() {
            self.sync_markets(Chain::HyperCore).await?;
        }
        Ok(self.should_connect_perpetuals(wallet))
    }

    pub async fn set_pinned(&self, perpetual_id: PerpetualId, name: String, pinned: bool) -> Result<GemToast, GemServiceError> {
        self.store.set_pinned(vec![perpetual_id], pinned).await?;
        Ok(GemToast::pinned(name, pinned))
    }

    pub async fn connection_address(&self, wallet: Wallet) -> Result<Option<String>, GemServiceError> {
        let Some(account) = hyperliquid_account(&wallet.accounts) else {
            return Ok(None);
        };
        let address = account.address.clone();
        self.update_account_mode(wallet.id, Chain::HyperCore, address.clone()).await?;
        Ok(Some(address))
    }
}

impl GemPerpetualService {
    fn should_connect_perpetuals(&self, wallet: Option<Wallet>) -> bool {
        wallet.is_some_and(|wallet| self.preferences.show_perpetuals(wallet.wallet_type, wallet.chains()))
    }

    pub async fn sync_markets_if_needed(&self, chain: Chain, trigger: GemMarketsRefreshTrigger) -> Result<bool, GemServiceError> {
        let _sync = self.markets_sync.lock().await;
        if !trigger.should_sync_markets(self.preferences.get_perpetual_markets_updated_at(), Utc::now().timestamp()) {
            return Ok(false);
        }
        self.sync_markets(chain).await?;
        Ok(true)
    }

    pub fn deposit_assets(&self, wallet_id: WalletId) -> Vec<Asset> {
        GemPerpetual::new(PerpetualProvider::Hypercore).deposit_assets(self.wallet_preferences.get_perpetual_account_mode(wallet_id))
    }

    async fn update_deposit_balances(&self) -> Result<(), GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        self.balance.update(wallet.id.clone(), self.deposit_assets(wallet.id).into_iter().map(|asset| asset.id).collect()).await
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
        let _writes = self.writes.lock().await;
        let stored = self.store.get_perpetuals(data.iter().map(|data| data.perpetual.name.clone()).collect()).await?;
        let changed = rules::changed_perpetuals(data, &stored);
        if changed.is_empty() {
            return Ok(());
        }
        self.store.save_perpetuals(changed).await
    }

    pub async fn get_portfolio(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<PerpetualPortfolio, GemServiceError> {
        let mode = self.wallet_preferences.get_perpetual_account_mode(wallet_id);
        Ok(self.gateway.get_perpetual_portfolio(chain, address, mode).await?)
    }

    pub async fn on_socket_message(&self, wallet_id: WalletId, data: Vec<u8>) -> Result<GemPerpetualSocketUpdate, GemServiceError> {
        let message = parse_websocket_data(&data).map_err(GemServiceError::core)?;
        match message {
            HyperliquidSocketMessage::AccountState { dex, balance, positions } => {
                if !rules::spot_is_perpetual_collateral(self.wallet_preferences.get_perpetual_account_mode(wallet_id.clone()))
                    && let Some(total) = self.total_balance(wallet_id.clone(), dex, balance)
                {
                    self.update_balance(wallet_id.clone(), total).await?;
                }
                let positions = self.with_stored_markets(positions).await?;
                let existing = self.store.get_positions(wallet_id.clone(), PerpetualProvider::Hypercore).await?;
                let diff = diff_clearinghouse_positions(dex, positions, existing);
                self.store.update_positions(wallet_id, diff.positions, diff.delete_position_ids).await?;
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::SpotState { balance } => {
                if rules::spot_is_perpetual_collateral(self.wallet_preferences.get_perpetual_account_mode(wallet_id.clone())) {
                    self.update_balance(wallet_id, balance).await?;
                }
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::OpenOrders { dex, orders } => {
                let existing = self.store.get_positions(wallet_id.clone(), PerpetualProvider::Hypercore).await?;
                let diff = diff_open_orders_positions(dex, &orders, existing);
                self.store.update_positions(wallet_id, diff.positions, diff.delete_position_ids).await?;
                Ok(GemPerpetualSocketUpdate::Applied)
            }
            HyperliquidSocketMessage::Candle { candle } => Ok(GemPerpetualSocketUpdate::Candle { candle }),
            HyperliquidSocketMessage::MarketData { market } => {
                let _writes = self.writes.lock().await;
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
        let PerpetualAccountPositions { mode, summary, is_complete } = self.gateway.get_positions(chain, address).await?;
        self.wallet_preferences.set_perpetual_account_mode(wallet_id.clone(), mode)?;
        if is_complete || rules::spot_is_perpetual_collateral(mode) {
            self.update_balance(wallet_id.clone(), summary.balance).await?;
        }
        let positions = self.with_stored_markets(summary.positions).await?;
        let delete_ids = match is_complete {
            true => stale(self.store.get_position_ids(wallet_id.clone(), provider(chain)?).await?, positions.iter().map(|position| position.id.clone())),
            false => Vec::new(),
        };
        self.store.update_positions(wallet_id, positions, delete_ids).await?;
        Ok(mode)
    }

    fn total_balance(&self, wallet_id: WalletId, dex: PerpDex, balance: PerpetualBalance) -> Option<PerpetualBalance> {
        let mut balances = self.dex_balances.lock().unwrap();
        balances.insert((wallet_id.clone(), dex), balance);
        let dexes = self.perp_dexes.iter().map(|dex| balances.get(&(wallet_id.clone(), *dex)).map(|balance| (*dex, balance.clone()))).collect::<Option<Vec<_>>>()?;
        Some(total_dexes_balance(dexes))
    }

    async fn with_stored_markets(&self, positions: Vec<PerpetualPosition>) -> Result<Vec<PerpetualPosition>, GemServiceError> {
        if positions.is_empty() {
            return Ok(positions);
        }
        let stored = self.store.get_perpetuals(positions.iter().map(|position| position.perpetual_id.symbol.clone()).collect()).await?;
        Ok(positions.into_iter().filter(|position| stored.iter().any(|perpetual| perpetual.id == position.perpetual_id)).collect())
    }

    async fn update_account_mode(&self, wallet_id: WalletId, chain: Chain, address: String) -> Result<(), GemServiceError> {
        match self.gateway.get_perpetual_account_mode(chain, address).await {
            Ok(mode) => self.wallet_preferences.set_perpetual_account_mode(wallet_id, mode),
            Err(_) => Ok(()),
        }
    }
}

impl GemPerpetualService {
    pub async fn get_candlesticks(&self, chain: Chain, symbol: String, period: ChartPeriod) -> Result<Vec<GemChartCandleStick>, GemServiceError> {
        Ok(self.gateway.get_perpetual_candlesticks(chain, symbol, period.as_ref().to_string()).await?)
    }

    pub async fn clear_markets(&self) -> Result<(), GemServiceError> {
        self.store.clear_perpetuals(rules::collateral_asset_ids()).await?;
        self.recent_activity.clear_in(GemRecentActivityScope::AllWallets, vec![RecentActivityType::Perpetual]).await?;
        self.preferences.set_perpetual_markets_updated_at(None)
    }
}

impl GemPerpetualService {
    pub async fn update_balance(&self, wallet_id: WalletId, balance: PerpetualBalance) -> Result<(), GemServiceError> {
        let update = rules::balance_update(&balance).map_err(GemServiceError::core)?;
        self.balance.update_balances(wallet_id, vec![update]).await
    }

    pub async fn update_prices(&self, prices: HashMap<String, f64>) -> Result<(), GemServiceError> {
        let _writes = self.writes.lock().await;
        let now = Utc::now().timestamp();
        let prices = {
            let mut pending = self.pending_prices.lock().unwrap();
            pending.extend(prices);
            if !is_outdated(self.preferences.get_perpetual_prices_updated_at(), now, PRICES_UPDATE_INTERVAL_SECONDS) {
                return Ok(());
            }
            std::mem::take(&mut *pending)
        };
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
    use std::sync::atomic::Ordering;

    use futures::executor::block_on;
    use primitives::Account;

    use super::testkit::{EMPTY_CLEARINGHOUSE_STATE, PerpetualTestkit};
    use super::*;
    use crate::services::balance::GemAssetBalance;
    use crate::services::transfer::{GemRecentActivity, GemRecentActivityStore};
    use crate::testkit::TestAlienProvider;
    use primitives::AssetType;
    use primitives::known_assets::{ARBITRUM_USDC, HYPERCORE_SPOT_USDC};

    const ALL_MIDS: &str = r#"{"channel":"allMids","data":{"mids":{"BTC":"104633.0","ETH":"3321.1"}}}"#;
    const OPEN_ORDERS: &str = r#"{"channel":"openOrders","data":{"dex":"","user":"0xc64c","orders":[{"coin":"BTC","oid":1,"triggerPx":"110000.0","limitPx":"110000.0","isPositionTpsl":true,"orderType":"Take Profit Market"}]}}"#;
    const CANDLE: &str = r#"{"channel":"candle","data":{"t":1700000000000,"T":1700000059999,"s":"BTC","i":"1m","o":"1.0","c":"2.0","h":"3.0","l":"0.5","v":"10.0","n":3}}"#;
    const SUBSCRIPTION: &str = r#"{"channel":"subscriptionResponse","data":{"method":"subscribe","subscription":{"type":"allMids"}}}"#;

    fn message(testkit: &PerpetualTestkit, payload: &str) -> GemPerpetualSocketUpdate {
        block_on(testkit.service.on_socket_message(testkit.wallet_id.clone(), payload.as_bytes().to_vec())).unwrap()
    }

    #[test]
    fn test_a_price_message_writes_once_and_then_waits_out_the_interval() {
        let testkit = PerpetualTestkit::new();
        *testkit.store.stored.lock().unwrap() = vec![primitives::perpetual::Perpetual {
            name: "BTC".to_string(),
            price: 100_000.0,
            ..primitives::perpetual::Perpetual::mock()
        }];

        assert_eq!(message(&testkit, ALL_MIDS), GemPerpetualSocketUpdate::Applied);
        assert_eq!(message(&testkit, ALL_MIDS), GemPerpetualSocketUpdate::Applied);

        let writes = testkit.store.price_writes.lock().unwrap();
        assert_eq!(writes.len(), 1, "a second message inside the interval waits for the next write");
        assert_eq!(writes[0].iter().map(|price| (price.coin.as_str(), price.price)).collect::<Vec<_>>(), vec![("BTC", 104_633.0)]);
        assert!(testkit.preferences.get_perpetual_prices_updated_at().is_some());
    }

    #[test]
    fn test_prices_from_every_dex_reach_the_next_write() {
        let testkit = PerpetualTestkit::new();
        *testkit.store.stored.lock().unwrap() = [("BTC", 100_000.0), ("xyz:NVDA", 200.0)]
            .map(|(name, price)| primitives::perpetual::Perpetual {
                name: name.to_string(),
                price,
                ..primitives::perpetual::Perpetual::mock()
            })
            .to_vec();

        message(&testkit, ALL_MIDS);
        message(&testkit, r#"{"channel":"allMids","data":{"dex":"xyz","mids":{"xyz:NVDA":"222.165"}}}"#);
        testkit.preferences.set_perpetual_prices_updated_at(Some(0)).unwrap();
        message(&testkit, ALL_MIDS);

        let written: Vec<Vec<(String, f64)>> = testkit.store.price_writes.lock().unwrap().iter().map(|prices| prices.iter().map(|price| (price.coin.clone(), price.price)).collect()).collect();
        assert_eq!(
            written,
            vec![vec![("BTC".to_string(), 104_633.0)], vec![("xyz:NVDA".to_string(), 222.165)]],
            "the xyz mids that arrive inside the interval go out with the next write instead of being dropped"
        );
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
        *testkit.store.stored.lock().unwrap() = vec![
            primitives::perpetual::Perpetual {
                name: "BTC".to_string(),
                price: 104_633.0,
                ..primitives::perpetual::Perpetual::mock()
            },
            primitives::perpetual::Perpetual {
                name: "ETH".to_string(),
                price: 3_000.0,
                ..primitives::perpetual::Perpetual::mock()
            },
        ];

        message(&testkit, ALL_MIDS);

        let writes = testkit.store.price_writes.lock().unwrap();
        assert_eq!(writes[0].iter().map(|price| price.coin.as_str()).collect::<Vec<_>>(), vec!["ETH"]);
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

        let update = block_on(testkit.service.on_socket_message(testkit.wallet_id.clone(), br#"{"channel":"somethingElse","data":{}}"#.to_vec()));

        assert!(
            matches!(update, Err(GemServiceError::Core { .. })),
            "only a bare unknown channel reads as Unknown; one carrying a payload is reported so a new Hyperliquid channel is noticed"
        );
    }

    #[test]
    fn test_an_account_snapshot_keeps_its_balance_when_its_positions_cannot_be_stored() {
        block_on(async {
            let mut testkit = PerpetualTestkit::with_unified_balance().await;
            testkit.service.perp_dexes = vec![PerpDex::Main];
            testkit.store.rejects_positions.store(true, Ordering::SeqCst);
            let snapshot = include_bytes!("../../../../crates/gem_hypercore/testdata/ws_clearinghouse_state.json");

            let update = testkit.service.on_socket_message(testkit.wallet_id.clone(), snapshot.to_vec()).await;

            assert!(update.is_err());
            assert_eq!(testkit.balances.balances.lock().unwrap()[&testkit.wallet_id][0].available.to_string(), "14400000000");
        })
    }

    #[test]
    fn test_a_refresh_missing_a_dex_keeps_that_dexs_positions_and_the_balance() {
        block_on(async {
            let testkit = PerpetualTestkit::with_provider(TestAlienProvider::with_json_by_request_type(&[
                ("userAbstraction", r#""default""#),
                ("clearinghouseState", EMPTY_CLEARINGHOUSE_STATE),
                ("clearinghouseState:xyz", "{}"),
            ]));
            let xyz = PerpetualPosition {
                id: "xyz:NVDA".to_string(),
                ..PerpetualPosition::mock()
            };
            *testkit.store.positions.lock().unwrap() = vec![xyz.clone()];

            testkit.service.sync_positions(testkit.wallet_id.clone(), Chain::HyperCore, "0x123".to_string()).await.unwrap();

            assert_eq!(*testkit.store.positions.lock().unwrap(), vec![xyz], "a dex that did not answer keeps its stored positions");
            assert!(testkit.balances.balances.lock().unwrap().get(&testkit.wallet_id).is_none(), "a balance missing a dex is not shown");
        })
    }

    #[test]
    fn test_a_dex_snapshot_keeps_the_other_dexes_positions_and_adds_up_the_balance() {
        let mut testkit = block_on(PerpetualTestkit::with_unified_balance());
        testkit.service.perp_dexes = vec![PerpDex::Main, PerpDex::Xyz];
        *testkit.store.stored.lock().unwrap() = ["ETH", "xyz:NVDA"]
            .map(|name| primitives::perpetual::Perpetual {
                id: PerpetualId::new(PerpetualProvider::Hypercore, name),
                name: name.to_string(),
                ..primitives::perpetual::Perpetual::mock()
            })
            .to_vec();
        let available = |testkit: &PerpetualTestkit| testkit.balances.balances.lock().unwrap().get(&testkit.wallet_id).map(|balances| balances[0].available.to_string());

        message(&testkit, include_str!("../../../../crates/gem_hypercore/testdata/ws_clearinghouse_state.json"));
        assert_eq!(available(&testkit), None, "the balance waits until every dex has reported");

        message(&testkit, include_str!("../../../../crates/gem_hypercore/testdata/ws_clearinghouse_state_xyz.json"));
        assert_eq!(available(&testkit), Some((14_400_000_000u64 + 900_000_000u64).to_string()));
        let withdrawable = testkit.balances.balances.lock().unwrap()[&testkit.wallet_id][0].withdrawable.to_string();
        assert_eq!(withdrawable, "14400000000", "Hyperliquid pays a withdrawal from the main dex only");
        let held: Vec<String> = testkit.store.positions.lock().unwrap().iter().map(|position| position.perpetual_id.symbol.clone()).collect();
        assert_eq!(held, vec!["ETH".to_string(), "xyz:NVDA".to_string()], "the xyz snapshot does not delete the main dex position");
    }

    #[test]
    fn test_the_stored_account_mode_picks_the_balance_message_that_counts() {
        let testkit = block_on(PerpetualTestkit::with_unified_balance());
        let available = |testkit: &PerpetualTestkit| testkit.balances.balances.lock().unwrap().get(&testkit.wallet_id).map(|balances| balances[0].available.to_string());
        let spot_state = include_str!("../../../../crates/gem_hypercore/testdata/ws_spot_state.json");

        message(&testkit, spot_state);
        assert_eq!(available(&testkit), None, "a standard account's spot USDC is not its perpetual balance");

        testkit.wallet_preferences.set_perpetual_account_mode(testkit.wallet_id.clone(), PerpetualAccountMode::Unified).unwrap();
        message(&testkit, include_str!("../../../../crates/gem_hypercore/testdata/ws_clearinghouse_state.json"));
        assert_eq!(available(&testkit), None, "a unified account's dex snapshot holds only the margin in use");

        message(&testkit, spot_state);
        assert_eq!(available(&testkit), Some("3797316".to_string()), "the switch to unified is read from the stored mode without a new subscription");
    }

    #[test]
    fn test_a_position_waits_for_its_market_without_holding_up_the_others() {
        let testkit = PerpetualTestkit::new();
        let snapshot = include_str!("../../../../crates/gem_hypercore/testdata/ws_clearinghouse_state.json");

        assert_eq!(message(&testkit, snapshot), GemPerpetualSocketUpdate::Applied);
        *testkit.store.stored.lock().unwrap() = vec![primitives::perpetual::Perpetual {
            id: PerpetualId::new(PerpetualProvider::Hypercore, "ETH"),
            name: "ETH".to_string(),
            ..primitives::perpetual::Perpetual::mock()
        }];
        assert_eq!(message(&testkit, snapshot), GemPerpetualSocketUpdate::Applied);

        let written: Vec<Vec<String>> = testkit
            .store
            .position_writes
            .lock()
            .unwrap()
            .iter()
            .map(|(positions, _)| positions.iter().map(|position| position.perpetual_id.symbol.clone()).collect())
            .collect();
        assert_eq!(written, vec![vec![], vec!["ETH".to_string()]], "a position whose market is not stored yet is left out until the market is");
    }

    #[test]
    fn test_a_position_refresh_keeps_its_balance_when_its_positions_cannot_be_stored() {
        block_on(async {
            let testkit = PerpetualTestkit::with_unified_balance().await;
            let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));
            testkit.select_wallet(&wallet);
            testkit.store.rejects_positions.store(true, Ordering::SeqCst);

            assert!(testkit.service.sync_current_positions().await.is_err());
            assert_eq!(testkit.balances.balances.lock().unwrap()[&wallet.id][0].available.to_string(), "12093224");
        })
    }

    #[test]
    fn test_a_payload_that_is_not_a_socket_message_is_an_error() {
        let testkit = PerpetualTestkit::new();

        let error = block_on(testkit.service.on_socket_message(testkit.wallet_id.clone(), b"not json".to_vec()));

        assert!(matches!(error, Err(GemServiceError::Core { .. })), "{error:?}");
    }

    #[test]
    fn test_disabled_perpetuals_clear_the_markets_without_asking_the_gateway() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_enabled(false).unwrap();

            assert!(!testkit.service.sync_enablement(None).await.unwrap());

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
    fn test_switching_perpetuals_off_forgets_perpetual_recents_in_every_wallet() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            let perpetual = Asset {
                asset_type: AssetType::PERPETUAL,
                ..Asset::from_chain(Chain::HyperCore)
            };
            let another_wallets_recent = GemRecentActivity {
                activity_type: RecentActivityType::Perpetual,
                asset_id: perpetual.id.clone(),
                to_asset_id: None,
            };
            testkit.service.add_recent(GemAssetAction::Open, perpetual).await.unwrap();
            testkit.service.add_recent(GemAssetAction::Open, Asset::from_chain(Chain::Ethereum)).await.unwrap();
            testkit.recents.add(another_wallets_recent, WalletId::Multicoin("other".to_string())).await.unwrap();
            testkit.preferences.set_perpetual_enabled(false).unwrap();

            testkit.service.sync_enablement(None).await.unwrap();

            let kept: Vec<RecentActivityType> = testkit.recents.added.lock().unwrap().iter().map(|(activity, _)| activity.activity_type.clone()).collect();
            assert_eq!(kept, vec![RecentActivityType::Search]);
        })
    }

    #[test]
    fn test_stale_markets_do_not_hold_up_the_connection() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            testkit.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp() - 24 * 60 * 60)).unwrap();
            let hypercore = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));

            assert!(testkit.service.sync_enablement(Some(hypercore)).await.unwrap());
            assert!(testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_markets_that_were_never_synced_load_before_the_connection() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            let hypercore = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));

            assert!(testkit.service.sync_enablement(Some(hypercore)).await.is_err(), "positions cannot be stored before their markets");
            assert!(!testkit.provider.requested_paths().is_empty());
        })
    }

    #[test]
    fn test_overlapping_enablement_syncs_load_the_markets_once() {
        block_on(async {
            let testkit = PerpetualTestkit::with_provider(TestAlienProvider::with_json_by_request_type(&[(
                "metaAndAssetCtxs",
                include_str!("../../../../crates/gem_hypercore/testdata/perpetual_markets_response_meta_and_asset_ctxs.json"),
            )]));
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            testkit.store.yields_while_saving.store(true, Ordering::SeqCst);

            let (first, second) = futures::join!(testkit.service.sync_enablement(None), testkit.service.sync_enablement(None));

            assert_eq!((first.unwrap(), second.unwrap()), (false, false));
            assert_eq!(testkit.provider.requested_types(), vec!["metaAndAssetCtxs"; testkit.service.perp_dexes.len()], "one sync asks each dex once");
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
            testkit.select_wallet(&wallet);

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
    fn test_connection_fetches_only_the_account_mode_before_opening_the_socket() {
        block_on(async {
            for chains in [[Chain::Arbitrum, Chain::HyperCore], [Chain::Hyperliquid, Chain::HyperCore], [Chain::HyperCore, Chain::Arbitrum]] {
                let testkit = PerpetualTestkit::with_unified_balance().await;
                let wallet = Wallet::mock_with_accounts(Account::mock_chains(&chains, "0xc64c"));

                let address = testkit.service.connection_address(wallet.clone()).await.unwrap();

                assert_eq!(address, Some("0xc64c".to_string()));
                assert_eq!(testkit.wallet_preferences.get_perpetual_account_mode(wallet.id), PerpetualAccountMode::Unified);
                assert_eq!(testkit.provider.requested_types(), vec!["userAbstraction"]);
                assert!(testkit.store.position_writes.lock().unwrap().is_empty());
                assert!(testkit.balances.balances.lock().unwrap().is_empty());
            }
        });
    }

    #[test]
    fn test_refresh_uses_hypercore_for_evm_accounts() {
        block_on(async {
            for chains in [[Chain::Arbitrum, Chain::HyperCore], [Chain::Hyperliquid, Chain::HyperCore], [Chain::HyperCore, Chain::Arbitrum]] {
                let testkit = PerpetualTestkit::with_unified_balance().await;
                let wallet = Wallet::mock_with_accounts(Account::mock_chains(&chains, "0xc64c"));
                testkit.select_wallet(&wallet);
                testkit.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp())).unwrap();

                let failures = testkit.service.refresh(GemMarketsRefreshTrigger::Scheduled).await;
                assert!(failures.iter().all(|failure| failure.step == GemPerpetualRefreshStep::DepositBalances), "only the Arbitrum deposit balance has no node here");

                let stored = testkit.balances.balances.lock().unwrap();
                assert_eq!(stored[&wallet.id][0].available.to_string(), "12093224");
                assert_eq!(stored[&wallet.id][0].withdrawable.to_string(), "12093224");
                assert_eq!(testkit.wallet_preferences.get_perpetual_account_mode(wallet.id), PerpetualAccountMode::Unified);
            }
        });
    }

    #[test]
    fn test_refresh_updates_the_balances_a_standard_account_deposits_from() {
        block_on(async {
            let testkit = PerpetualTestkit::with_standard_spot_balance().await;
            let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));
            testkit.select_wallet(&wallet);
            testkit.preferences.set_perpetual_markets_updated_at(Some(Utc::now().timestamp())).unwrap();

            assert_eq!(testkit.service.refresh(GemMarketsRefreshTrigger::Scheduled).await, vec![]);

            let stored = testkit.balances.balances.lock().unwrap();
            let spot = stored[&wallet.id].iter().find(|balance| balance.asset_id == HYPERCORE_SPOT_USDC.id).map(|balance| balance.available.to_string());
            assert_eq!(spot, Some("307393500".to_string()), "the deposit picker lists spot USDC even when the wallet never enabled it");
        });
    }

    #[test]
    fn test_connection_and_refresh_each_ask_for_the_account_mode_once() {
        block_on(async {
            let testkit = PerpetualTestkit::with_unified_balance().await;
            let wallet = Wallet::mock_with_accounts(Account::mock_chains(&[Chain::HyperCore], "0xc64c"));
            testkit.select_wallet(&wallet);

            testkit.service.connection_address(wallet).await.unwrap();
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
    fn test_deposit_target_offers_spot_usdc_only_to_a_standard_account() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            let funded = |asset: &Asset| GemAssetBalance {
                asset_id: asset.id.clone(),
                ..GemAssetBalance::mock_with_available(5_000_000)
            };
            testkit.balances.balances.lock().unwrap().insert(testkit.wallet_id.clone(), vec![funded(&ARBITRUM_USDC), funded(&HYPERCORE_SPOT_USDC)]);

            assert_eq!(testkit.service.deposit_target().await.unwrap(), GemPerpetualDepositTarget::SelectAsset);

            testkit.wallet_preferences.set_perpetual_account_mode(testkit.wallet_id.clone(), PerpetualAccountMode::Unified).unwrap();

            assert_eq!(
                testkit.service.deposit_target().await.unwrap(),
                GemPerpetualDepositTarget::Amount { asset: ARBITRUM_USDC.clone() },
                "a unified account's spot USDC already is its perpetual balance"
            );
        })
    }

    #[test]
    fn test_an_unreachable_account_mode_falls_back_to_the_stored_one() {
        block_on(async {
            let testkit = PerpetualTestkit::new();
            testkit.wallet_preferences.set_perpetual_account_mode(testkit.wallet_id.clone(), PerpetualAccountMode::Unified).unwrap();

            testkit.service.update_account_mode(testkit.wallet_id.clone(), Chain::HyperCore, "0xc64c".to_string()).await.unwrap();

            assert_eq!(testkit.wallet_preferences.get_perpetual_account_mode(testkit.wallet_id.clone()), PerpetualAccountMode::Unified);
        })
    }

    #[test]
    fn test_an_account_with_no_stored_mode_reads_as_standard() {
        block_on(async {
            let testkit = PerpetualTestkit::new();

            testkit.service.update_account_mode(testkit.wallet_id.clone(), Chain::HyperCore, "0xc64c".to_string()).await.unwrap();

            assert_eq!(testkit.wallet_preferences.get_perpetual_account_mode(testkit.wallet_id.clone()), PerpetualAccountMode::Standard);
        })
    }
}
