use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeDelta, Utc};
use gem_tracing::info_with_fields;
use localizer::LanguageLocalizer;
use number_formatter::NumberFormatter;
use prices::{PriceAlertNotification, PriceAlertRules};
use primitives::{AssetId, PriceAlert, PriceAlertType, PriceAlerts};
use push_notification::{GorushNotification, PushNotification, PushNotificationAsset, PushNotificationTypes};

use super::repository::Repository;

#[derive(Clone)]
pub struct PriceAlertClient {
    repository: Arc<dyn Repository>,
}

impl PriceAlertClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn get_price_alerts(&self, device_id: &str, asset_id: Option<&AssetId>) -> Result<PriceAlerts, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.device_price_alerts(device_id.to_string(), asset_id.cloned()).await?)
    }

    pub async fn add_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.add_price_alerts(device_id.to_string(), price_alerts).await?)
    }

    pub async fn delete_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = price_alerts.iter().map(PriceAlert::id).collect::<HashSet<_>>().into_iter().collect();
        Ok(self.repository.delete_price_alerts(device_id.to_string(), ids).await?)
    }

    pub async fn get_devices_to_alert(&self, rules: PriceAlertRules, primary_price_max_age: Duration) -> Result<Vec<PriceAlertNotification>, Box<dyn Error + Send + Sync>> {
        let now = Utc::now();
        let cooldown = TimeDelta::seconds(rules.notification_cooldown.as_secs() as i64);
        Ok(self.repository.notify_price_alerts(rules, (now - cooldown).naive_utc(), now.naive_utc(), primary_price_max_age).await?)
    }

    pub fn get_notifications_for_price_alerts(&self, notifications: Vec<PriceAlertNotification>) -> Vec<GorushNotification> {
        let formatter = NumberFormatter::new();
        let mut results = vec![];

        for alert in notifications {
            let Some(current_price) = formatter.currency(alert.price.price, alert.currency.as_ref()) else {
                info_with_fields!("unknown_currency_symbol", currency = alert.currency.as_ref());
                continue;
            };

            let change = formatter.percent(alert.price.price_change_percentage_24h);
            let localizer = LanguageLocalizer::new_with_language(alert.device.locale.as_ref());
            let asset_name = alert.asset.full_name();

            let message = match alert.alert_type {
                PriceAlertType::PriceUp | PriceAlertType::PriceDown | PriceAlertType::PriceMilestone => {
                    let Some(target_price) = alert.target.and_then(|value| formatter.currency(value, alert.currency.as_ref())) else {
                        continue;
                    };
                    localizer.price_alert_target(&asset_name, &target_price, &current_price, &change)
                }
                PriceAlertType::PriceChangesUp | PriceAlertType::PricePercentChangeUp => localizer.price_alert_up(&asset_name, &current_price, &change),
                PriceAlertType::PriceChangesDown | PriceAlertType::PricePercentChangeDown => localizer.price_alert_down(&asset_name, &current_price, &change),
                PriceAlertType::AllTimeHigh => localizer.price_alert_all_time_high(&alert.asset.name, &current_price),
            };

            let data = PushNotification {
                data: serde_json::to_value(&PushNotificationAsset { asset_id: alert.asset.id.clone() }).ok(),
                notification_type: PushNotificationTypes::PriceAlert,
            };

            results.extend(GorushNotification::from_device(alert.device, message.title, message.description, data));
        }

        results
    }
}
