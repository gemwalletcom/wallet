use primitives::{Asset, Currency, Price, PriceAlert, PriceAlertData};

use crate::models::state::GemLoadState;
use crate::services::price_alert::rules::{self, GemAssetPriceAlerts, GemPriceAlertList};

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

    pub fn list(&self, alerts: Vec<PriceAlertData>, price_currency: Currency, state: GemLoadState) -> GemPriceAlertList {
        rules::price_alert_list(alerts, price_currency, state)
    }

    pub fn asset_alerts(&self, asset: Asset, price: Option<Price>, alerts: Vec<PriceAlertData>, price_currency: Currency, state: GemLoadState) -> GemAssetPriceAlerts {
        rules::asset_price_alerts(asset, price, alerts, price_currency, state)
    }
}
