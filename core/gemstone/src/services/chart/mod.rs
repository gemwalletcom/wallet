pub mod rules;

use std::sync::Arc;

use chrono::Utc;
use primitives::currency::Currency;
use primitives::{AssetId, Chain, ChartDateValue, ChartPeriod};

use crate::api::{GemApiClient, GemApiError};
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
use crate::services::price_alert::GemPriceAlertService;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChart {
    pub values: Vec<ChartDateValue>,
    pub current: Option<ChartDateValue>,
}

#[derive(uniffi::Object)]
pub struct GemChartService {
    api: Arc<GemApiClient>,
    price: Arc<GemPriceService>,
    preferences: Arc<GemPreferencesService>,
    price_alerts: Arc<GemPriceAlertService>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemChartService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemApiClient>,
        price: Arc<GemPriceService>,
        preferences: Arc<GemPreferencesService>,
        price_alerts: Arc<GemPriceAlertService>,
        explorer: Arc<GemExplorerService>,
    ) -> Self {
        Self {
            api,
            price,
            preferences,
            price_alerts,
            explorer,
        }
    }

    pub fn token_url(&self, chain: Chain, address: String) -> Option<BlockExplorerLink> {
        self.explorer.get_token_url(chain, address)
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn chart_period(&self) -> ChartPeriod {
        self.preferences.get_chart_period()
    }

    pub fn set_chart_period(&self, period: ChartPeriod) -> Result<(), GemServiceError> {
        self.preferences.set_chart_period(period)
    }

    pub async fn sync_charts(&self, asset_id: AssetId, period: ChartPeriod) -> Result<GemChart, GemServiceError> {
        let currency = self.get_currency();
        let charts = self.api.client.get_charts(asset_id.clone(), period).await.map_err(GemApiError::from)?;
        if let Some(market) = charts.market {
            self.price.update_market(asset_id.clone(), market, currency.clone()).await?;
        }
        let rate = self.price.rate(currency.clone()).await?.ok_or(GemServiceError::InvalidInput {
            msg: format!("unknown currency: {currency}"),
        })?;
        let latest = self.price.prices(vec![asset_id]).await?.into_iter().next();
        let values = rules::converted_values(charts.prices, rate.rate);
        let current = rules::current_value(&values, latest, Utc::now());
        Ok(GemChart { values, current })
    }

    pub async fn sync_price_alerts(&self, asset_id: AssetId) -> Result<(), GemServiceError> {
        self.price_alerts.sync(Some(asset_id)).await
    }
}
