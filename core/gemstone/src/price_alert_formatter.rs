use number_formatter::price_suggestion;
use primitives::{PriceAlert, PriceAlertNotificationType};

use crate::services::price_alert::rules::sorted_price_alerts;

#[derive(Default, uniffi::Object)]
pub struct PriceAlertFormatter {}

#[uniffi::export]
impl PriceAlertFormatter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn percentage_suggestions(&self, price: f64) -> Vec<i32> {
        price_suggestion::percentage_suggestions(price)
    }

    pub fn rounded_values(&self, price: f64, by_percent: f64) -> Vec<f64> {
        price_suggestion::price_rounded_values(price, by_percent)
    }

    pub fn alert_id(&self, alert: PriceAlert) -> String {
        alert.id()
    }

    pub fn notification_type(&self, alert: PriceAlert) -> PriceAlertNotificationType {
        alert.notification_type()
    }

    pub fn should_display(&self, alert: PriceAlert) -> bool {
        alert.should_display()
    }

    pub fn sorted_alerts(&self, alerts: Vec<PriceAlert>) -> Vec<PriceAlert> {
        sorted_price_alerts(alerts)
    }
}
