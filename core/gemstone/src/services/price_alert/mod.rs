pub mod rules;
pub mod session;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{AssetId, Currency, PriceAlert};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::amount::model::GemNumberFormat;
use crate::services::banner::GemNotificationPermissions;
use crate::services::preferences::GemPreferencesService;
use session::GemPriceAlertSession;

pub use store::GemPriceAlertStore;

#[derive(uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
    preferences: Arc<GemPreferencesService>,
    store: Arc<dyn GemPriceAlertStore>,
    permissions: Arc<dyn GemNotificationPermissions>,
}

#[uniffi::export]
impl GemPriceAlertService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, preferences: Arc<GemPreferencesService>, store: Arc<dyn GemPriceAlertStore>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        Self { api, preferences, store, permissions }
    }

    pub fn is_enabled(&self) -> bool {
        self.preferences.is_price_alerts_enabled()
    }

    pub async fn set_enabled(&self, enabled: bool) -> Result<(), GemServiceError> {
        if self.is_enabled() == enabled {
            return Ok(());
        }
        if enabled {
            if !self.permissions.request_permissions_or_open_settings().await? {
                return Ok(());
            }
            self.preferences.set_push_notifications_enabled(true)?;
        }
        self.preferences.set_price_alerts_enabled(enabled)
    }

    pub fn new_alert_session(&self, asset_id: AssetId, format: GemNumberFormat) -> GemPriceAlertSession {
        GemPriceAlertSession::new(asset_id, self.get_currency(), format)
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub async fn enable_price_alert(&self, alert: PriceAlert) -> Result<(), GemServiceError> {
        self.add_price_alerts(vec![alert]).await?;
        self.set_enabled(true).await
    }

    pub async fn set_auto_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        let alert = PriceAlert::new_auto(asset_id, self.get_currency());
        match enabled {
            true => self.enable_price_alert(alert).await,
            false => self.delete_price_alerts(vec![alert]).await,
        }
    }

    pub async fn refresh(&self, asset_id: Option<AssetId>, has_alerts: bool) -> GemLoadState {
        GemLoadState::refreshed(self.sync(asset_id).await, has_alerts)
    }

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.store.update_price_alerts(Vec::new(), alerts.iter().map(|alert| alert.id()).collect()).await?;
        match self.api.client.delete_price_alerts(alerts.clone()).await {
            Ok(()) => Ok(()),
            Err(error) => {
                self.store.update_price_alerts(alerts, Vec::new()).await?;
                Err(GemApiError::from(error).into())
            }
        }
    }
}

impl GemPriceAlertService {
    pub async fn price_alerts(&self, asset_id: Option<AssetId>) -> Result<Vec<PriceAlert>, GemServiceError> {
        self.store.get_price_alerts(asset_id).await
    }

    pub async fn sync(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        let remote = self.api.client.get_price_alerts(asset_id.as_ref().map(ToString::to_string)).await.map_err(GemApiError::from)?;
        let remote = match &asset_id {
            Some(asset_id) => remote.into_iter().filter(|alert| alert.asset_id == *asset_id).collect(),
            None => remote,
        };
        let local = self.store.get_price_alerts(asset_id).await?;
        let changes = rules::reconcile(local, remote);
        if changes.delete_ids.is_empty() && changes.alerts.is_empty() {
            return Ok(());
        }
        self.store.update_price_alerts(changes.alerts, changes.delete_ids).await
    }

    pub async fn add_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.store.update_price_alerts(alerts.clone(), Vec::new()).await?;
        match self.api.client.add_price_alerts(alerts.clone()).await {
            Ok(()) => Ok(()),
            Err(error) => {
                self.store.update_price_alerts(Vec::new(), alerts.iter().map(|alert| alert.id()).collect()).await?;
                Err(GemApiError::from(error).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::rules::reconcile;
    use super::testkit::{GrantedNotificationPermissions, PriceAlertTestkit};
    use crate::services::banner::testkit::DeniedNotificationPermissions;
    use crate::services::error::GemServiceError;
    use crate::services::price_alert::store::GemPriceAlertStore;
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use primitives::{Chain, PriceAlert};
    use std::sync::Arc;

    #[test]
    fn test_set_enabled_asks_for_permission_and_writes_nothing_the_device_has_to_be_told() {
        let kit = PriceAlertTestkit::with_provider(Arc::new(TestAlienProvider::offline()), Arc::new(GrantedNotificationPermissions));
        assert_eq!(block_on(kit.service.set_enabled(true)), Ok(()));
        assert!(kit.service.is_enabled());
        assert!(kit.provider.requested_paths().is_empty(), "the device record is compared, never announced");

        let denied = PriceAlertTestkit::with_provider(Arc::new(TestAlienProvider::offline()), Arc::new(DeniedNotificationPermissions));
        assert_eq!(block_on(denied.service.set_enabled(true)), Ok(()));
        assert!(!denied.service.is_enabled());
    }

    #[test]
    fn test_an_alert_the_api_refused_is_not_left_behind_for_the_next_reconciliation() {
        let alert = PriceAlert::mock(Chain::Bitcoin, Some(1.0));

        let added = PriceAlertTestkit::with_provider(Arc::new(TestAlienProvider::offline()), Arc::new(GrantedNotificationPermissions));
        assert_eq!(block_on(added.service.add_price_alerts(vec![alert.clone()])), Err(GemServiceError::Offline));
        assert!(added.store.identifiers().is_empty());

        let deleted = PriceAlertTestkit::with_provider(Arc::new(TestAlienProvider::offline()), Arc::new(GrantedNotificationPermissions));
        block_on(deleted.store.update_price_alerts(vec![alert.clone()], Vec::new())).unwrap();
        assert_eq!(block_on(deleted.service.delete_price_alerts(vec![alert.clone()])), Err(GemServiceError::Offline));
        assert_eq!(deleted.store.identifiers(), vec![alert.id()]);
    }

    #[test]
    fn test_a_store_that_cannot_write_never_reaches_the_api() {
        let kit = PriceAlertTestkit::with_provider(Arc::new(TestAlienProvider::offline()), Arc::new(GrantedNotificationPermissions));
        let error = GemServiceError::Store { msg: "disk full".to_string() };
        *kit.store.write_error.lock().unwrap() = Some(error.clone());

        assert_eq!(block_on(kit.service.add_price_alerts(vec![PriceAlert::mock(Chain::Bitcoin, Some(1.0))])), Err(error));
        assert!(kit.provider.requested_paths().is_empty());
    }

    #[test]
    fn test_reconcile() {
        let local = vec![PriceAlert::mock(Chain::Bitcoin, None), PriceAlert::mock(Chain::Ethereum, None)];
        let remote = vec![PriceAlert::mock(Chain::Bitcoin, None), PriceAlert::mock(Chain::Solana, Some(1.0))];
        let changes = reconcile(local.clone(), remote.clone());
        assert_eq!(changes.delete_ids, vec![local[1].id()]);
        assert_eq!(changes.alerts.iter().map(PriceAlert::id).collect::<Vec<_>>(), vec![remote[1].id()]);

        let unchanged = reconcile(local.clone(), local.clone());
        assert!(unchanged.delete_ids.is_empty() && unchanged.alerts.is_empty());

        let changes = reconcile(Vec::new(), Vec::new());
        assert!(changes.delete_ids.is_empty() && changes.alerts.is_empty());
    }
}
