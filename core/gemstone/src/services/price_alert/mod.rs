pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{AssetId, Currency, PriceAlert};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::banner::GemNotificationPermissions;
use crate::services::device::GemDeviceService;
use crate::services::preferences::GemPreferencesService;

pub use store::GemPriceAlertStore;

#[derive(uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
    preferences: Arc<GemPreferencesService>,
    store: Arc<dyn GemPriceAlertStore>,
    device: Arc<GemDeviceService>,
    permissions: Arc<dyn GemNotificationPermissions>,
}

#[uniffi::export]
impl GemPriceAlertService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        preferences: Arc<GemPreferencesService>,
        store: Arc<dyn GemPriceAlertStore>,
        device: Arc<GemDeviceService>,
        permissions: Arc<dyn GemNotificationPermissions>,
    ) -> Self {
        Self {
            api,
            preferences,
            store,
            device,
            permissions,
        }
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
        self.preferences.set_price_alerts_enabled(enabled)?;
        self.device.synchronize().await.map(|_| ())
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub async fn enable_price_alert(&self, alert: PriceAlert) -> Result<(), GemServiceError> {
        self.add_price_alerts(vec![alert]).await?;
        self.set_enabled(true).await
    }

    pub async fn set_auto_alert(&self, asset_id: AssetId, enabled: bool) -> Result<(), GemServiceError> {
        let alert = PriceAlert::new_auto(asset_id, self.currency());
        match enabled {
            true => self.enable_price_alert(alert).await,
            false => self.delete_price_alerts(vec![alert]).await,
        }
    }

    pub async fn sync(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        let remote = self
            .api
            .client
            .get_price_alerts(asset_id.as_ref().map(ToString::to_string))
            .await
            .map_err(GemApiError::from)?;
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

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.store.update_price_alerts(Vec::new(), alerts.iter().map(|alert| alert.id()).collect()).await?;
        self.api.client.delete_price_alerts(alerts).await.map_err(GemApiError::from)?;
        Ok(())
    }
}

impl GemPriceAlertService {
    pub async fn add_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.store.update_price_alerts(alerts.clone(), Vec::new()).await?;
        self.api.client.add_price_alerts(alerts).await.map_err(GemApiError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::rules::reconcile;
    use primitives::currency::Currency;
    use primitives::{AssetId, Chain, PriceAlert};

    fn alert(chain: Chain, price: Option<f64>) -> PriceAlert {
        PriceAlert {
            asset_id: AssetId::from_chain(chain),
            currency: Currency::USD,
            price,
            price_percent_change: None,
            price_direction: None,
            last_notified_at: None,
            identifier: String::new(),
        }
    }

    #[test]
    fn test_reconcile() {
        let local = vec![alert(Chain::Bitcoin, None), alert(Chain::Ethereum, None)];
        let remote = vec![alert(Chain::Bitcoin, None), alert(Chain::Solana, Some(1.0))];
        let changes = reconcile(local.clone(), remote.clone());
        assert_eq!(changes.delete_ids, vec![local[1].id()]);
        assert_eq!(changes.alerts.iter().map(PriceAlert::id).collect::<Vec<_>>(), vec![remote[1].id()]);

        let unchanged = reconcile(local.clone(), local.clone());
        assert!(unchanged.delete_ids.is_empty() && unchanged.alerts.is_empty());

        let changes = reconcile(Vec::new(), Vec::new());
        assert!(changes.delete_ids.is_empty() && changes.alerts.is_empty());
    }
}
