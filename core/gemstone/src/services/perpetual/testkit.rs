use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use num_bigint::BigInt;
use primitives::known_assets::HYPERCORE_PERPETUAL_USDC;
use primitives::perpetual::PerpetualData;
use primitives::{Asset, AssetBasic, AssetId, AssetProperties, AssetScore, AutocloseValidation, PerpetualDirection, PerpetualMarginType, PerpetualMarketData, PerpetualPosition, PerpetualProvider, TpslType, Wallet, WalletId};

use super::model::{GemPerpetualOrderAction, GemPerpetualOrderInput, GemPerpetualTransferData};
use super::{GemAutocloseField, GemAutocloseModify, GemPerpetualService, GemPerpetualStore};
use crate::gateway::GemGateway;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::assets::{GemAssetStore, GemAssetsService};
use crate::services::balance::GemBalanceService;
use crate::services::balance::testkit::MemoryBalanceStore;
use crate::services::error::GemServiceError;
use crate::services::node::GemNodeService;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::price::GemPriceService;
use crate::services::price::testkit::MemoryPriceStore;
use crate::services::stream::testkit::SubscriptionTestkit;
use crate::services::transfer::GemRecentActivityService;
use crate::services::transfer::testkit::MemoryRecentActivityStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_preferences::testkit::MemoryWalletPreferencesStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

#[derive(Default)]
pub struct MemoryPerpetualStore {
    pub positions: Mutex<Vec<PerpetualPosition>>,
    pub position_writes: Mutex<Vec<(Vec<PerpetualPosition>, Vec<String>)>>,
    pub markets: Mutex<Vec<PerpetualMarketData>>,
    pub price_writes: Mutex<Vec<HashMap<String, f64>>>,
    pub deleted: Mutex<u32>,
    pub cleared_collateral: Mutex<Vec<Vec<AssetId>>>,
    pub perpetual_writes: Mutex<Vec<Vec<PerpetualData>>>,
    pub pin_writes: Mutex<Vec<(Vec<String>, bool)>>,
}

#[async_trait]
impl GemPerpetualStore for MemoryPerpetualStore {
    async fn save_perpetuals(&self, perpetuals: Vec<PerpetualData>) -> Result<(), GemServiceError> {
        self.perpetual_writes.lock().unwrap().push(perpetuals);
        Ok(())
    }
    async fn set_pinned(&self, ids: Vec<String>, pinned: bool) -> Result<(), GemServiceError> {
        self.pin_writes.lock().unwrap().push((ids, pinned));
        Ok(())
    }
    async fn clear_perpetuals(&self, collateral_asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        *self.deleted.lock().unwrap() += 1;
        self.cleared_collateral.lock().unwrap().push(collateral_asset_ids);
        Ok(())
    }
    async fn get_positions(&self, _: WalletId, _: PerpetualProvider) -> Result<Vec<PerpetualPosition>, GemServiceError> {
        Ok(self.positions.lock().unwrap().clone())
    }
    async fn get_position_ids(&self, _: WalletId, _: PerpetualProvider) -> Result<Vec<String>, GemServiceError> {
        Ok(self.positions.lock().unwrap().iter().map(|position| position.id.clone()).collect())
    }
    async fn update_positions(&self, _: WalletId, positions: Vec<PerpetualPosition>, delete_ids: Vec<String>) -> Result<(), GemServiceError> {
        self.position_writes.lock().unwrap().push((positions, delete_ids));
        Ok(())
    }
    async fn update_market(&self, market: PerpetualMarketData) -> Result<(), GemServiceError> {
        self.markets.lock().unwrap().push(market);
        Ok(())
    }
    async fn update_prices(&self, prices: HashMap<String, f64>) -> Result<(), GemServiceError> {
        self.price_writes.lock().unwrap().push(prices);
        Ok(())
    }
}

pub struct PerpetualTestkit {
    pub service: GemPerpetualService,
    pub provider: Arc<TestAlienProvider>,
    pub store: Arc<MemoryPerpetualStore>,
    pub asset_store: Arc<MemoryAssetStore>,
    pub wallets: Arc<MemoryWalletStore>,
    pub balances: Arc<MemoryBalanceStore>,
    pub preferences: Arc<GemPreferencesService>,
    pub wallet_preferences: Arc<GemWalletPreferencesService>,
    pub wallet_id: WalletId,
}

impl PerpetualTestkit {
    pub fn new() -> Self {
        Self::with_provider(TestAlienProvider::with_status(503))
    }

    pub async fn with_unified_balance() -> Self {
        let testkit = Self::with_provider(TestAlienProvider::with_json_by_request_type(&[
            ("userAbstraction", r#""unifiedAccount""#),
            (
                "clearinghouseState",
                r#"{"assetPositions":[],"marginSummary":{"accountValue":"0","totalNtlPos":"0","totalRawUsd":"0","totalMarginUsed":"0"},"crossMarginSummary":{"accountValue":"0","totalNtlPos":"0","totalRawUsd":"0","totalMarginUsed":"0"},"crossMaintenanceMarginUsed":"0","withdrawable":"0"}"#,
            ),
            (
                "spotClearinghouseState",
                r#"{"balances":[{"coin":"USDC","token":0,"total":"12.093224","hold":"0","entryNtl":"0"}],"tokenToAvailableAfterMaintenance":[[0,"12.093224"]]}"#,
            ),
        ]));
        testkit
            .asset_store
            .save_assets(vec![AssetBasic::new(HYPERCORE_PERPETUAL_USDC.clone(), AssetProperties::default(HYPERCORE_PERPETUAL_USDC.id.clone()), AssetScore::new(0))])
            .await
            .unwrap();
        testkit
    }

    fn with_provider(provider: TestAlienProvider) -> Self {
        let wallet = Wallet::mock();
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let wallets = Arc::new(MemoryWalletStore {
            wallets: Mutex::new(vec![wallet.clone()]),
            ..Default::default()
        });
        let session = Arc::new(GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore {
                current: Mutex::new(Some(wallet.id.clone())),
            }),
            wallets.clone(),
        ));
        let provider = Arc::new(provider);
        let gateway = Arc::new(GemGateway::new(provider.clone(), Arc::new(GemNodeService::mock()), preferences_store, Arc::new(EmptyPreferences)));
        let price = Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default())));
        let asset_store = Arc::new(MemoryAssetStore::default());
        let assets = Arc::new(GemAssetsService::mock(provider.clone(), asset_store.clone()));
        let balances = Arc::new(MemoryBalanceStore::default());
        let balance = Arc::new(GemBalanceService::new(
            gateway.clone(),
            wallets.clone(),
            asset_store.clone(),
            balances.clone(),
            assets.clone(),
            Arc::new(SubscriptionTestkit::new(&[], &[]).service),
        ));
        let wallet_preferences = Arc::new(GemWalletPreferencesService::new(Arc::new(MemoryWalletPreferencesStore::default())));
        let store = Arc::new(MemoryPerpetualStore::default());
        let service = GemPerpetualService::new(
            gateway,
            price,
            store.clone(),
            assets,
            preferences.clone(),
            balance,
            wallet_preferences.clone(),
            session.clone(),
            Arc::new(GemRecentActivityService::new(Arc::new(MemoryRecentActivityStore::default()), session)),
        );
        Self {
            service,
            provider,
            store,
            asset_store,
            wallets,
            balances,
            preferences,
            wallet_preferences,
            wallet_id: wallet.id,
        }
    }
}

impl Default for PerpetualTestkit {
    fn default() -> Self {
        Self::new()
    }
}

impl GemPerpetualOrderInput {
    pub fn mock(action: GemPerpetualOrderAction) -> Self {
        Self {
            action,
            direction: PerpetualDirection::Long,
            margin_type: PerpetualMarginType::Cross,
            base_asset: Asset::mock(),
            asset: Asset::mock(),
            asset_index: 1,
            price: 100.0,
            usdc_value: BigInt::from(50_000_000),
            usdc_decimals: 6,
            leverage: 4,
            slippage: None,
            take_profit: None,
            stop_loss: None,
        }
    }
}

impl GemPerpetualTransferData {
    pub fn mock() -> Self {
        Self {
            provider: PerpetualProvider::Hypercore,
            direction: PerpetualDirection::Long,
            asset: Asset::mock(),
            base_asset: Asset::mock(),
            asset_index: 0,
            price: 100.0,
            leverage: 3,
            margin_type: PerpetualMarginType::Cross,
        }
    }
}

impl GemAutocloseField {
    pub fn mock(price: Option<f64>, original_price: Option<f64>, is_valid: bool, order_id: Option<u64>) -> Self {
        Self {
            tpsl_type: TpslType::TakeProfit,
            price,
            original_price,
            formatted_price: price.map(|price| format!("{price:.1}")),
            validation: match is_valid {
                true => AutocloseValidation::Valid,
                false => AutocloseValidation::InvalidAmount,
            },
            order_id,
        }
    }
}

impl GemAutocloseModify {
    pub fn mock(take_profit: GemAutocloseField, stop_loss: GemAutocloseField) -> Self {
        Self {
            direction: PerpetualDirection::Long,
            asset_index: Some(5),
            take_profit,
            stop_loss,
        }
    }
}
