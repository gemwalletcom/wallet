use number_formatter::price_suggestion;
use primitives::{PriceAlert, PriceAlertDirection, PriceAlertNotificationType};

use crate::services::price_alert::rules;

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

    pub fn displayed_alert_ids(&self, alerts: Vec<PriceAlert>) -> Vec<String> {
        rules::displayed_price_alert_ids(alerts)
    }

    pub fn alert_direction(
        &self,
        notification_type: PriceAlertNotificationType,
        input_value: Option<f64>,
        current_price: Option<f64>,
        selected_direction: PriceAlertDirection,
    ) -> Option<PriceAlertDirection> {
        rules::alert_direction(notification_type, input_value, current_price, selected_direction)
    }
}
