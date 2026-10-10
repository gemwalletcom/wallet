use async_trait::async_trait;
use primitives::{AssetId, Chain, Currency, PriceAlert, Wallet};
use std::sync::{Arc, Mutex};

use super::session::GemPriceAlertSession;
use super::store::GemPriceAlertStore;
use crate::api::GemDeviceApiClient;
use crate::services::assets::GemAssetsService;
use crate::services::assets::testkit::MemoryAssetStore;
use crate::services::banner::GemNotificationPermissions;
use crate::services::device::{GemDeviceKeyService, GemDeviceService};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use crate::services::wallet::testkit::MemoryWalletStore;
use crate::services::wallet_session::GemWalletSessionService;
use crate::services::wallet_session::testkit::MemoryWalletSessionStore;
use crate::testkit::{EmptyPreferences, TestAlienProvider};

impl GemPriceAlertSession {
    pub fn mock() -> Self {
        Self::new(AssetId::from_chain(Chain::Ethereum), Currency::USD, crate::services::amount::model::GemNumberFormat { decimal_separator: ".".to_string() }).on_price(Some(100.0), None)
    }
}

#[derive(Default)]
pub struct MemoryPriceAlertStore {
    pub alerts: Mutex<Vec<PriceAlert>>,
    pub write_error: Mutex<Option<GemServiceError>>,
}

impl MemoryPriceAlertStore {
    pub fn with_alerts(alerts: Vec<PriceAlert>) -> Self {
        Self {
            alerts: Mutex::new(alerts),
            write_error: Mutex::new(None),
        }
    }

    pub fn identifiers(&self) -> Vec<String> {
        self.alerts.lock().unwrap().iter().map(PriceAlert::id).collect()
    }
}

#[async_trait]
impl GemPriceAlertStore for MemoryPriceAlertStore {
    async fn get_price_alerts(&self, _: Option<AssetId>) -> Result<Vec<PriceAlert>, GemServiceError> {
        Ok(self.alerts.lock().unwrap().clone())
    }

    async fn update_price_alerts(&self, alerts: Vec<PriceAlert>, delete_ids: Vec<String>) -> Result<(), GemServiceError> {
        if let Some(error) = self.write_error.lock().unwrap().clone() {
            return Err(error);
        }
        let mut stored = self.alerts.lock().unwrap();
        stored.retain(|alert| !delete_ids.contains(&alert.id()));
        for alert in alerts {
            match stored.iter().position(|stored| stored.id() == alert.id()) {
                Some(index) => stored[index] = alert,
                None => stored.push(alert),
            }
        }
        Ok(())
    }
}

pub struct GrantedNotificationPermissions;

#[async_trait]
impl GemNotificationPermissions for GrantedNotificationPermissions {
    fn is_available(&self) -> bool {
        true
    }
    async fn request_permissions_or_open_settings(&self) -> Result<bool, GemServiceError> {
        Ok(true)
    }
}

pub fn price_alert_service(store: Arc<dyn GemPriceAlertStore>) -> Arc<super::GemPriceAlertService> {
    let provider = Arc::new(TestAlienProvider::new(crate::alien::AlienResponse::new(None, Vec::new())));
    Arc::new(price_alert_service_with(provider, store, Arc::new(MemoryAssetStore::default()), Arc::new(GrantedNotificationPermissions)))
}

fn price_alert_service_with(provider: Arc<TestAlienProvider>, store: Arc<dyn GemPriceAlertStore>, asset_store: Arc<MemoryAssetStore>, permissions: Arc<dyn GemNotificationPermissions>) -> super::GemPriceAlertService {
    let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
    let api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
    let wallets = Arc::new(MemoryWalletStore {
        wallets: Mutex::new(vec![Wallet::mock()]),
        ..Default::default()
    });
    let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore::default()), wallets));
    let device = GemDeviceService::mock(api.clone(), session, preferences.clone());
    let assets = Arc::new(GemAssetsService::mock(provider, asset_store));
    super::GemPriceAlertService::new(api, preferences, store, assets, device, permissions)
}

pub struct PriceAlertTestkit {
    pub service: super::GemPriceAlertService,
    pub store: Arc<MemoryPriceAlertStore>,
    pub asset_store: Arc<MemoryAssetStore>,
    pub provider: Arc<TestAlienProvider>,
}

impl PriceAlertTestkit {
    pub fn with_provider(provider: Arc<TestAlienProvider>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        let store = Arc::new(MemoryPriceAlertStore::default());
        let asset_store = Arc::new(MemoryAssetStore::default());
        let service = price_alert_service_with(provider.clone(), store.clone(), asset_store.clone(), permissions);
        Self { service, store, asset_store, provider }
    }
}
