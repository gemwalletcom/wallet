pub mod error;
pub mod rules;
pub mod store;

use std::sync::Arc;

use primitives::PriceAlert;

use crate::api::{GemApiError, GemDeviceApiClient};

pub use error::GemPriceAlertError;
pub use store::GemPriceAlertStore;

#[derive(uniffi::Object)]
pub struct GemPriceAlertService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemPriceAlertStore>,
}

#[uniffi::export]
impl GemPriceAlertService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemPriceAlertStore>) -> Self {
        Self { api, store }
    }

    pub async fn sync(&self, asset_id: Option<String>) -> Result<(), GemPriceAlertError> {
        let remote = self.api.client.get_price_alerts(asset_id.clone()).await.map_err(GemApiError::from)?;
        let remote = match &asset_id {
            Some(asset_id) => remote.into_iter().filter(|alert| alert.asset_id.to_string() == *asset_id).collect(),
            None => remote,
        };
        let local = self.store.get_price_alerts(asset_id).await?;
        let changes = rules::reconcile(local, remote);
        if changes.delete_ids.is_empty() && changes.alerts.is_empty() {
            return Ok(());
        }
        self.store.apply(changes.delete_ids, changes.alerts).await
    }

    pub async fn add_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemPriceAlertError> {
        Ok(self.api.client.add_price_alerts(alerts).await.map_err(GemApiError::from)?)
    }

    pub async fn delete_price_alerts(&self, alerts: Vec<PriceAlert>) -> Result<(), GemPriceAlertError> {
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
