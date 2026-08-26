pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{AssetId, PriceAlert};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::preferences::GemPreferencesService;

pub use store::GemPriceAlertStore;

#[derive(uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
    preferences: Arc<GemPreferencesService>,
    store: Arc<dyn GemPriceAlertStore>,
}

#[uniffi::export]
impl GemPriceAlertService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, preferences: Arc<GemPreferencesService>, store: Arc<dyn GemPriceAlertStore>) -> Self {
        Self { api, preferences, store }
    }

    pub fn is_enabled(&self) -> Result<bool, GemServiceError> {
        self.preferences.is_price_alerts_enabled()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<(), GemServiceError> {
        self.preferences.set_price_alerts_enabled(enabled)
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
        self.store.update(changes.alerts, changes.delete_ids).await
    }

    pub async fn add_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.store.update(alerts.clone(), Vec::new()).await?;
        Ok(self.api.client.add_price_alerts(alerts).await.map_err(GemApiError::from)?)
    }

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemServiceError> {
        self.store.update(Vec::new(), alerts.iter().map(|alert| alert.id()).collect()).await?;
        Ok(self.api.client.delete_price_alerts(alerts).await.map_err(GemApiError::from)?)
    }

    pub fn price_alert_id(&self, alert: PriceAlert) -> String {
        alert.id()
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
        assert_eq!(changes.alerts.len(), 2);

        let changes = reconcile(Vec::new(), Vec::new());
        assert!(changes.delete_ids.is_empty() && changes.alerts.is_empty());
    }
}
