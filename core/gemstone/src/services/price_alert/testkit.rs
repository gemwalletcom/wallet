use async_trait::async_trait;
use primitives::{AssetId, Chain, Currency, PriceAlert};
use std::sync::{Arc, Mutex};

use super::session::GemPriceAlertSession;
use super::store::GemPriceAlertStore;
use crate::api::GemDeviceApiClient;
use crate::services::banner::GemNotificationPermissions;
use crate::services::device::GemDeviceKeyService;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
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
    Arc::new(super::GemPriceAlertService::new(
        Arc::new(GemDeviceApiClient::new(provider, Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences))))),
        Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))),
        store,
        Arc::new(GrantedNotificationPermissions),
    ))
}

pub struct PriceAlertTestkit {
    pub service: super::GemPriceAlertService,
    pub store: Arc<MemoryPriceAlertStore>,
    pub provider: Arc<TestAlienProvider>,
}

impl PriceAlertTestkit {
    pub fn with_provider(provider: Arc<TestAlienProvider>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        let store = Arc::new(MemoryPriceAlertStore::default());
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        let api = Arc::new(GemDeviceApiClient::new(provider.clone(), Arc::new(GemDeviceKeyService::new(Arc::new(EmptyPreferences)))));
        let service = super::GemPriceAlertService::new(api, preferences, store.clone(), permissions);
        Self { service, store, provider }
    }
}
