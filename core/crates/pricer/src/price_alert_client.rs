use chrono::{Duration, Utc};
use gem_tracing::info_with_fields;
use localizer::LanguageLocalizer;
use number_formatter::NumberFormatter;
use primitives::currency::Currency;
use primitives::{
    Asset, AssetId, Device, FiatRate, GorushNotification, Price, PriceAlert, PriceAlertDirection, PriceAlertType, PriceAlerts, PriceData, PushNotification, PushNotificationAsset,
    PushNotificationTypes,
};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration as StdDuration;
use storage::{AssetsRepository, Database, PriceAlertsRepository};

const DEFAULT_RANK: i32 = 1000;

#[derive(Clone)]
pub struct PriceAlertClient {
    database: Database,
}

#[derive(Clone, Debug)]
pub struct PriceAlertNotification {
    pub device: Device,
    pub asset: Asset,
    pub price: Price,
    pub alert_type: PriceAlertType,
    pub price_alert: PriceAlert,
    pub milestone: Option<f64>,
}

impl PriceAlertNotification {
    pub fn currency(&self) -> &Currency {
        match self.alert_type {
            PriceAlertType::PriceUp | PriceAlertType::PriceDown => &self.price_alert.currency,
            PriceAlertType::PriceChangesUp
            | PriceAlertType::PriceChangesDown
            | PriceAlertType::PricePercentChangeUp
            | PriceAlertType::PricePercentChangeDown
            | PriceAlertType::AllTimeHigh
            | PriceAlertType::PriceMilestone => &self.device.currency,
        }
    }

    fn with_rates(self, rates: &[FiatRate]) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let rate = fiat_rate(rates, self.currency()).ok_or_else(|| format!("missing fiat rate for {}", self.currency().as_ref()))?;
        Ok(Self {
            price: self.price.with_rate(rate),
            ..self
        })
    }
}

#[derive(Clone, Debug)]
pub struct PriceAlertRules {
    pub notification_cooldown: StdDuration,
    pub price_change_threshold: f64,
    pub rank_divisor: f64,
    pub milestones: Vec<f64>,
}

#[derive(Debug, PartialEq)]
struct AlertResult {
    alert_type: PriceAlertType,
    milestone: Option<f64>,
}

impl AlertResult {
    fn new(alert_type: PriceAlertType) -> Self {
        Self { alert_type, milestone: None }
    }

    fn with_milestone(alert_type: PriceAlertType, milestone: f64) -> Self {
        Self {
            alert_type,
            milestone: Some(milestone),
        }
    }
}

impl PriceAlertRules {
    fn calculate_threshold(&self, rank: i32) -> f64 {
        let rank = if rank > 0 { rank } else { DEFAULT_RANK };
        self.price_change_threshold * (1.0 + (rank as f64).ln() / self.rank_divisor)
    }

    fn find_crossed_milestone(&self, price_24h_ago: f64, current_price: f64) -> Option<f64> {
        if current_price <= price_24h_ago {
            return None;
        }

        self.milestones.iter().find(|&&milestone| price_24h_ago < milestone && current_price >= milestone).copied()
    }
}

fn fiat_rate(rates: &[FiatRate], currency: &Currency) -> Option<f64> {
    rates.iter().find(|rate| rate.symbol == *currency).map(|rate| rate.rate)
}

fn calculate_price_24h_ago(current_price: f64, change_percent: f64) -> f64 {
    let divisor = 1.0 + change_percent / 100.0;
    if divisor <= 0.0 {
        return current_price;
    }
    current_price / divisor
}

impl PriceAlertClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_price_alerts(&self, device_id: &str, asset_id: Option<&AssetId>) -> Result<PriceAlerts, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .price_alerts()?
            .get_price_alerts_for_device_id(device_id, asset_id)?
            .into_iter()
            .map(|x| x.price_alert)
            .collect())
    }

    pub async fn add_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.price_alerts()?.add_price_alerts(device_id, price_alerts)?)
    }

    pub async fn delete_price_alerts(&self, device_id: &str, price_alerts: PriceAlerts) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ids = price_alerts.iter().map(|x| x.id()).collect::<HashSet<_>>().into_iter().collect();
        Ok(self.database.price_alerts()?.delete_price_alerts(device_id, ids)?)
    }

    pub async fn get_devices_to_alert(&self, rules: PriceAlertRules, max_age: StdDuration) -> Result<Vec<PriceAlertNotification>, Box<dyn Error + Send + Sync>> {
        let now = Utc::now();
        let cooldown = Duration::seconds(rules.notification_cooldown.as_secs() as i64);
        let after_notified_at = now - cooldown;
        let price_alerts = self.database.price_alerts()?.get_price_alerts(after_notified_at.naive_utc(), max_age)?;
        let rates = self.fiat_rates()?;

        let mut results: Vec<PriceAlertNotification> = Vec::new();
        let mut price_alert_ids: HashSet<String> = HashSet::new();

        for (price_alert, price_data, device) in price_alerts {
            if let Some(alert_result) = Self::get_price_alert_type(&price_alert, &price_data, &rates, &rules) {
                let notification = self.price_alert_notification(device, &price_data, price_alert.clone(), alert_result, &rates)?;
                price_alert_ids.insert(price_alert.id());
                results.push(notification);
            }
        }

        self.database
            .price_alerts()?
            .update_price_alerts_set_notified_at(price_alert_ids.into_iter().collect(), now.naive_utc())?;
        Ok(results)
    }

    fn fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.fiat()?.get_fiat_rates()?.into_iter().map(|row| row.as_primitive()).collect())
    }

    fn get_price_alert_type(price_alert: &PriceAlert, price_data: &PriceData, rates: &[FiatRate], rules: &PriceAlertRules) -> Option<AlertResult> {
        // User-defined price target
        if let Some(target_price) = price_alert.price {
            let direction = price_alert.price_direction.clone()?;
            let price = price_data.price * fiat_rate(rates, &price_alert.currency)?;
            let alert_type = match direction {
                PriceAlertDirection::Up if price >= target_price => Some(PriceAlertType::PriceUp),
                PriceAlertDirection::Down if price <= target_price => Some(PriceAlertType::PriceDown),
                _ => None,
            };
            return alert_type.map(AlertResult::new);
        }

        // User-defined percent change
        if let Some(target_percent) = price_alert.price_percent_change {
            let direction = price_alert.price_direction.clone()?;
            let alert_type = match direction {
                PriceAlertDirection::Up if price_data.price_change_percentage_24h >= target_percent => Some(PriceAlertType::PricePercentChangeUp),
                PriceAlertDirection::Down if price_data.price_change_percentage_24h <= -target_percent => Some(PriceAlertType::PricePercentChangeDown),
                _ => None,
            };
            return alert_type.map(AlertResult::new);
        }

        // All-time high check
        if price_data.all_time_high > 0.0 && price_data.price > price_data.all_time_high {
            return Some(AlertResult::new(PriceAlertType::AllTimeHigh));
        }

        // Price milestone check
        let price_24h_ago = calculate_price_24h_ago(price_data.price, price_data.price_change_percentage_24h);
        if let Some(milestone) = rules.find_crossed_milestone(price_24h_ago, price_data.price) {
            return Some(AlertResult::with_milestone(PriceAlertType::PriceMilestone, milestone));
        }

        // Dynamic threshold based on rank
        let threshold = rules.calculate_threshold(price_data.market_cap_rank.unwrap_or(0));
        if price_data.price_change_percentage_24h > threshold {
            return Some(AlertResult::new(PriceAlertType::PriceChangesUp));
        }
        if price_data.price_change_percentage_24h < -threshold {
            return Some(AlertResult::new(PriceAlertType::PriceChangesDown));
        }

        None
    }

    fn price_alert_notification(
        &self,
        device: Device,
        price_data: &PriceData,
        price_alert: PriceAlert,
        alert_result: AlertResult,
        rates: &[FiatRate],
    ) -> Result<PriceAlertNotification, Box<dyn Error + Send + Sync>> {
        PriceAlertNotification {
            device,
            asset: self.database.assets()?.get_asset(&price_alert.asset_id)?,
            price: Price::new(price_data.price, price_data.price_change_percentage_24h, price_data.last_updated_at, price_data.provider),
            alert_type: alert_result.alert_type,
            price_alert,
            milestone: alert_result.milestone,
        }
        .with_rates(rates)
    }

    pub fn get_notifications_for_price_alerts(&self, notifications: Vec<PriceAlertNotification>) -> Vec<GorushNotification> {
        let mut results = vec![];
        let formatter = NumberFormatter::new();

        for alert in notifications {
            if !alert.device.can_receive_price_alerts() {
                continue;
            }

            let current_price = match formatter.currency(alert.price.price, alert.currency().as_ref()) {
                Some(p) => p,
                None => {
                    info_with_fields!("unknown_currency_symbol", currency = alert.currency().as_ref());
                    continue;
                }
            };

            let change = formatter.percent(alert.price.price_change_percentage_24h, alert.device.locale.as_ref());
            let localizer = LanguageLocalizer::new_with_language(alert.device.locale.as_ref());

            let message = match &alert.alert_type {
                PriceAlertType::PriceUp | PriceAlertType::PriceDown | PriceAlertType::PriceMilestone => {
                    let Some(target_value) = Self::price_alert_target_value(&alert) else {
                        continue;
                    };
                    let Some(target_price) = formatter.currency(target_value, alert.currency().as_ref()) else {
                        continue;
                    };
                    localizer.price_alert_target(&alert.asset.full_name(), &target_price, &current_price, &change)
                }
                PriceAlertType::PriceChangesUp | PriceAlertType::PricePercentChangeUp => localizer.price_alert_up(&alert.asset.full_name(), &current_price, &change),
                PriceAlertType::PriceChangesDown | PriceAlertType::PricePercentChangeDown => localizer.price_alert_down(&alert.asset.full_name(), &current_price, &change),
                PriceAlertType::AllTimeHigh => localizer.price_alert_all_time_high(&alert.asset.name, &current_price),
            };

            let data = PushNotification {
                data: serde_json::to_value(&PushNotificationAsset { asset_id: alert.asset.id.clone() }).ok(),
                notification_type: PushNotificationTypes::PriceAlert,
            };

            results.extend(GorushNotification::from_device(alert.device.clone(), message.title, message.description, data));
        }

        results
    }

    fn price_alert_target_value(alert: &PriceAlertNotification) -> Option<f64> {
        match &alert.alert_type {
            PriceAlertType::PriceUp | PriceAlertType::PriceDown => alert.price_alert.price,
            PriceAlertType::PriceMilestone => alert.milestone,
            PriceAlertType::PriceChangesUp
            | PriceAlertType::PriceChangesDown
            | PriceAlertType::PricePercentChangeUp
            | PriceAlertType::PricePercentChangeDown
            | PriceAlertType::AllTimeHigh => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use primitives::{Chain, PriceProvider};

    use super::*;

    fn rules() -> PriceAlertRules {
        PriceAlertRules {
            notification_cooldown: StdDuration::from_secs(86_400),
            price_change_threshold: 5.0,
            rank_divisor: 5.0,
            milestones: vec![],
        }
    }

    fn rates() -> Vec<FiatRate> {
        vec![
            FiatRate { symbol: Currency::USD, rate: 1.0 },
            FiatRate {
                symbol: Currency::EUR,
                rate: 0.86,
            },
        ]
    }

    #[test]
    fn test_get_price_alert_type() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let price_data = PriceData::mock_with(78_987.0, -1.4);
        let over = PriceAlert::new_price(asset_id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Up);
        let under = PriceAlert::new_price(asset_id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Down);
        let over_usd = PriceAlert::new_price(asset_id.clone(), Currency::USD, 71_000.0, PriceAlertDirection::Up);
        let over_unknown = PriceAlert::new_price(asset_id.clone(), Currency::JPY, 71_000.0, PriceAlertDirection::Up);
        let auto = PriceAlert::new_auto(asset_id, Currency::EUR);

        assert_eq!(PriceAlertClient::get_price_alert_type(&over, &price_data, &rates(), &rules()), None);
        assert_eq!(
            PriceAlertClient::get_price_alert_type(&under, &price_data, &rates(), &rules()),
            Some(AlertResult::new(PriceAlertType::PriceDown))
        );
        assert_eq!(
            PriceAlertClient::get_price_alert_type(&over_usd, &price_data, &rates(), &rules()),
            Some(AlertResult::new(PriceAlertType::PriceUp))
        );
        assert_eq!(PriceAlertClient::get_price_alert_type(&over_unknown, &price_data, &rates(), &rules()), None);
        assert_eq!(
            PriceAlertClient::get_price_alert_type(&auto, &PriceData::mock_with(78_987.0, 6.0), &[], &rules()),
            Some(AlertResult::new(PriceAlertType::PriceChangesUp))
        );
    }

    #[test]
    fn test_with_rates() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let price = Price::new(78_987.0, -1.4, DateTime::from_timestamp(1_788_821_182, 0).unwrap(), PriceProvider::Coingecko);
        let target = PriceAlertNotification {
            device: Device::mock(),
            asset: asset.clone(),
            price,
            alert_type: PriceAlertType::PriceUp,
            price_alert: PriceAlert::new_price(asset.id.clone(), Currency::EUR, 71_000.0, PriceAlertDirection::Up),
            milestone: None,
        };
        let automatic = PriceAlertNotification {
            alert_type: PriceAlertType::PriceChangesUp,
            price_alert: PriceAlert::new_auto(asset.id, Currency::EUR),
            ..target.clone()
        };

        assert_eq!(target.currency(), &Currency::EUR);
        assert_eq!(target.clone().with_rates(&rates()).unwrap().price, price.with_rate(0.86));
        assert_eq!(automatic.currency(), &Currency::USD);
        assert_eq!(automatic.with_rates(&rates()).unwrap().price, price);
        assert!(target.with_rates(&[]).is_err());
    }

    #[test]
    fn test_price_alert_target_value() {
        let asset = Asset::from_chain(Chain::Bitcoin);
        let alert = PriceAlertNotification {
            device: Device::mock(),
            asset: asset.clone(),
            price: Price::new(80_954.0, -0.27, Utc::now(), PriceProvider::Coingecko),
            alert_type: PriceAlertType::PriceDown,
            price_alert: PriceAlert::new_price(asset.id.clone(), Currency::USD, 81_000.0, PriceAlertDirection::Down),
            milestone: None,
        };

        assert_eq!(PriceAlertClient::price_alert_target_value(&alert), Some(81_000.0));

        let milestone_alert = PriceAlertNotification {
            alert_type: PriceAlertType::PriceMilestone,
            price_alert: PriceAlert::new_auto(asset.id, Currency::USD),
            milestone: Some(100_000.0),
            ..alert.clone()
        };
        assert_eq!(PriceAlertClient::price_alert_target_value(&milestone_alert), Some(100_000.0));

        let automatic_alert = PriceAlertNotification {
            alert_type: PriceAlertType::PriceChangesUp,
            ..alert
        };
        assert_eq!(PriceAlertClient::price_alert_target_value(&automatic_alert), None);
    }
}
