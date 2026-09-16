use number_formatter::price_suggestion;
use primitives::{Currency, PriceAlert, PriceAlertData, PriceAlertDirection, PriceAlertNotificationType};

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

    pub fn alert_id(&self, alert: PriceAlert) -> String {
        alert.id()
    }

    pub fn notification_type(&self, alert: PriceAlert) -> PriceAlertNotificationType {
        alert.notification_type()
    }

    pub fn alert_kind(&self, alert: PriceAlert) -> GemPriceAlertKind {
        rules::alert_kind(&alert)
    }

    pub fn row(&self, data: PriceAlertData, price_currency: Currency) -> GemPriceAlertRow {
        rules::price_alert_row(&data, price_currency)
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
