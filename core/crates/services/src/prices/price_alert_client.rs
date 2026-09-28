use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;

use chrono::{TimeDelta, Utc};
use gem_tracing::info_with_fields;
use localizer::LanguageLocalizer;
use number_formatter::NumberFormatter;
use prices::{PriceAlertNotification, PriceAlertRules, PriceAlertTrigger};
use primitives::{AssetId, Device, FiatRate, Price, PriceAlert, PriceAlertType, PriceAlerts, PriceData};
use push_notification::{GorushNotification, PushNotification, PushNotificationAsset, PushNotificationTypes};
use storage::{AssetsRepository, Database, DatabaseClient, FiatRepository, PriceAlertsRepository};

#[derive(Clone)]
pub struct PriceAlertClient {
    database: Database,
}

impl PriceAlertClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_price_alerts(&self, device_id: &str, asset_id: Option<&AssetId>) -> Result<PriceAlerts, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let asset_id = asset_id.cloned();
        let rows = self.database.run(move |client| client.get_price_alerts_for_device_id(&device_id, asset_id.as_ref())).await?;
        Ok(rows.into_iter().map(|row| row.price_alert).collect())
    }

    pub async fn add_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        Ok(self.database.run(move |client| client.add_price_alerts(&device_id, price_alerts)).await?)
    }

    pub async fn delete_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let ids = price_alerts.iter().map(PriceAlert::id).collect::<HashSet<_>>().into_iter().collect();
        Ok(self.database.run(move |client| client.delete_price_alerts(&device_id, ids)).await?)
    }

    pub async fn get_devices_to_alert(&self, rules: PriceAlertRules, primary_price_max_age: Duration) -> Result<Vec<PriceAlertNotification>, Box<dyn Error + Send + Sync>> {
        self.database
            .run(move |client| -> Result<_, Box<dyn Error + Send + Sync>> {
                let now = Utc::now();
                let cooldown = TimeDelta::seconds(rules.notification_cooldown.as_secs() as i64);
                let price_alerts = client.get_price_alerts((now - cooldown).naive_utc(), primary_price_max_age)?;
                let rates = client.get_fiat_rates()?;

                let mut notifications = Vec::new();
                let mut notified_ids = HashSet::new();
                for (price_alert, price_data, device) in price_alerts {
                    let Some(trigger) = rules.evaluate(&price_alert, &price_data, &rates) else {
                        continue;
                    };
                    notified_ids.insert(price_alert.id());
                    notifications.push(Self::notification(client, device, &price_data, price_alert, trigger, &rates)?);
                }

                client.update_price_alerts_set_notified_at(notified_ids.into_iter().collect(), now.naive_utc())?;
                Ok(notifications)
            })
            .await
    }

    fn notification(client: &mut DatabaseClient, device: Device, price_data: &PriceData, price_alert: PriceAlert, trigger: PriceAlertTrigger, rates: &[FiatRate]) -> Result<PriceAlertNotification, Box<dyn Error + Send + Sync>> {
        PriceAlertNotification {
            device,
            asset: client.get_asset(&price_alert.asset_id)?,
            price: Price::new(price_data.price, price_data.price_change_percentage_24h, price_data.last_updated_at, price_data.provider),
            alert_type: trigger.alert_type,
            price_alert,
            milestone: trigger.milestone,
        }
        .with_rates(rates)
    }

    pub fn get_notifications_for_price_alerts(&self, notifications: Vec<PriceAlertNotification>) -> Vec<GorushNotification> {
        let formatter = NumberFormatter::new();
        let mut results = vec![];

        for alert in notifications {
            if !alert.device.can_receive_price_alerts() {
                continue;
            }

            let Some(current_price) = formatter.currency(alert.price.price, alert.currency().as_ref()) else {
                info_with_fields!("unknown_currency_symbol", currency = alert.currency().as_ref());
                continue;
            };

            let change = formatter.percent(alert.price.price_change_percentage_24h, alert.device.locale.as_ref());
            let localizer = LanguageLocalizer::new_with_language(alert.device.locale.as_ref());
            let asset_name = alert.asset.full_name();

            let message = match alert.alert_type {
                PriceAlertType::PriceUp | PriceAlertType::PriceDown | PriceAlertType::PriceMilestone => {
                    let Some(target_price) = alert.target_value().and_then(|value| formatter.currency(value, alert.currency().as_ref())) else {
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
