use primitives::{Currency, PriceAlert, PriceAlertData};

use crate::services::price_alert::rules::{self, GemPriceAlertListSection, GemPriceAlertRow};

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

    pub fn row(&self, data: PriceAlertData, price_currency: Currency) -> GemPriceAlertRow {
        rules::price_alert_row(&data, price_currency)
    }

    pub fn sections(&self, alerts: Vec<PriceAlertData>, price_currency: Currency) -> Vec<GemPriceAlertListSection> {
        rules::price_alert_list_sections(alerts, price_currency)
    }
}
