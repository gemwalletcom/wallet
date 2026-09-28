use primitives::{Asset, Currency, Price, PriceAlert, PriceAlertData};

use crate::services::price_alert::rules::{self, GemAssetPriceAlerts, GemPriceAlertListSection};

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

    pub fn sections(&self, alerts: Vec<PriceAlertData>, price_currency: Currency) -> Vec<GemPriceAlertListSection> {
        rules::price_alert_list_sections(alerts, price_currency)
    }

    pub fn asset_alerts(&self, asset: Asset, price: Option<Price>, alerts: Vec<PriceAlertData>, price_currency: Currency) -> GemAssetPriceAlerts {
        rules::asset_price_alerts(asset, price, alerts, price_currency)
    }
}
