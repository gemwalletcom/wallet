pub mod model;
pub mod rules;
pub mod session;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use primitives::currency::Currency;
use primitives::{Asset, AssetId, AssetLink, AssetMarket, ChartDateValue, ChartPeriod, PriceAlert};

use crate::api::{GemApiClient, GemApiError};
use crate::models::list::GemListSection;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
use session::GemChartSession;

pub use model::{GemChartBounds, GemChartData, GemChartHeader, GemChartValueType};

#[uniffi::export]
pub fn candlestick_header(base: f64, value: f64) -> GemChartHeader {
    rules::series_header(GemChartValueType::Price, base, false, &Currency::USD, value, None)
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChart {
    pub values: Vec<ChartDateValue>,
    pub base_value: f64,
    pub current: Option<GemChartCurrent>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartCurrent {
    pub date: DateTime<Utc>,
    pub value: f64,
    pub change_percentage: f64,
}

#[derive(uniffi::Object)]
pub struct GemChartService {
    api: Arc<GemApiClient>,
    price: Arc<GemPriceService>,
    preferences: Arc<GemPreferencesService>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemChartService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, price: Arc<GemPriceService>, preferences: Arc<GemPreferencesService>, explorer: Arc<GemExplorerService>) -> Self {
        Self { api, price, preferences, explorer }
    }

    pub async fn sections(&self, asset: Asset, price: Option<f64>, market: Option<AssetMarket>, price_alerts: Vec<PriceAlert>, links: Vec<AssetLink>) -> Result<Vec<GemListSection>, GemServiceError> {
        let currency = self.preferences.get_currency();
        let market = match market {
            Some(market) => self.price.market_in_currency(market, currency.clone()).await?,
            None => None,
        };
        let contract_explorer = asset.id.token_id.clone().and_then(|token_id| self.explorer.get_token_url(asset.id.chain, token_id));
        Ok(rules::chart_sections(&asset, currency, price, market.as_ref(), price_alerts, links, contract_explorer))
    }

    pub fn new_session(&self) -> GemChartSession {
        GemChartSession::new(self.chart_period(), self.preferences.get_currency())
    }

    pub fn set_chart_period(&self, period: ChartPeriod) -> Result<(), GemServiceError> {
        self.preferences.set_chart_period(period)
    }

    pub async fn sync_charts(&self, asset_id: AssetId, period: ChartPeriod) -> Result<GemChart, GemServiceError> {
        let currency = self.preferences.get_currency();
        let charts = self.api.client.get_charts(asset_id.clone(), period).await.map_err(GemApiError::from)?;
        if let Some(market) = charts.market {
            self.price.update_market(asset_id.clone(), market).await?;
        }
        let rate = self.price.rate(currency.clone()).await?.ok_or(GemServiceError::InvalidInput {
            msg: format!("unknown currency: {currency}"),
        })?;
        let latest = self.price.prices(vec![asset_id]).await?.into_iter().next();
        let values = rules::converted_values(charts.prices, rate.rate);
        let base_value = rules::base_value(&values);
        let current = rules::current_value(&values, latest, Utc::now(), period, base_value);
        Ok(GemChart { values, base_value, current })
    }
}

impl GemChartService {
    pub fn chart_period(&self) -> ChartPeriod {
        self.preferences.get_chart_period()
    }
}
