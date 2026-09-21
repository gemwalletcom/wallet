use primitives::{Currency, PriceAlert, PriceAlertData, PriceAlertNotificationType};

use crate::services::price_alert::rules::{self, GemPriceAlertKind, GemPriceAlertRow, GemPriceAlertSection};

#[derive(Default, uniffi::Object)]
pub struct PriceAlertFormatter {}

#[uniffi::export]
impl PriceAlertFormatter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
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

    pub fn sections(&self, alerts: Vec<PriceAlertData>) -> Vec<GemPriceAlertSection> {
        rules::price_alert_sections(alerts)
    }
}
