use number_formatter::price_suggestion;
use primitives::{Currency, PriceAlert, PriceAlertDirection, PriceAlertNotificationType};

use crate::services::price_alert::rules::{self, GemPriceAlertKind, GemPriceAlertRow};

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

    pub fn alert_kind(&self, alert: PriceAlert) -> GemPriceAlertKind {
        rules::alert_kind(&alert)
    }

    pub fn row(&self, alert: PriceAlert, current_price: Option<f64>, price_change_percentage_24h: Option<f64>, price_currency: Currency) -> GemPriceAlertRow {
        rules::price_alert_row(&alert, current_price, price_change_percentage_24h, price_currency)
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
