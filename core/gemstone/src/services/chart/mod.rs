pub mod model;
mod points;
pub mod rules;
pub mod session;
#[cfg(test)]
pub(crate) mod testkit;
pub mod zoom;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use primitives::currency::Currency;
use primitives::{AssetId, ChartDateValue, ChartPeriod};

use crate::api::{GemApiClient, GemApiError};
use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::{GemPriceService, rules as price_rules};
use crate::services::price_alert::GemPriceAlertService;
use session::{GemChartInput, GemChartRequest, GemChartResult, GemChartSession, GemChartView};

pub use model::{GemChartBounds, GemChartData, GemChartHeader, GemChartValueType};
pub use zoom::GemChartZoom;

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
    price_alerts: Arc<GemPriceAlertService>,
}

#[uniffi::export]
impl GemChartService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, price: Arc<GemPriceService>, preferences: Arc<GemPreferencesService>, explorer: Arc<GemExplorerService>, price_alerts: Arc<GemPriceAlertService>) -> Self {
        Self {
            api,
            price,
            preferences,
            explorer,
            price_alerts,
        }
    }

    pub fn new_session(&self) -> GemChartSession {
        GemChartSession::new(self.preferences.get_chart_period(), self.preferences.get_currency())
    }

    pub fn view_state(&self, session: GemChartSession, input: GemChartInput) -> GemChartView {
        let market = match (input.market.as_ref(), session.rate()) {
            (Some(market), Some(rate)) => Some(price_rules::market_in_currency(market.clone(), rate)),
            _ => None,
        };
        let contract_explorer = input.asset.id.token_id.clone().and_then(|token_id| self.explorer.get_token_url(input.asset.id.chain, token_id));
        GemChartView {
            period: session.period,
            phase: session.phase(input.asset_price()),
            is_refreshing: session.is_refreshing,
            sections: rules::chart_sections(
                &input.asset,
                session.currency,
                input.price.map(|price| price.price),
                market.as_ref(),
                self.price_alerts.is_available().then_some(input.price_alerts),
                input.links,
                contract_explorer,
            ),
        }
    }

    pub async fn load(&self, asset_id: AssetId, request: GemChartRequest) -> GemChartResult {
        match request.clone() {
            GemChartRequest::Rate { currency } => {
                let rate = self.rate(currency).await;
                GemChartResult {
                    request,
                    state: GemLoadState::of(&rate),
                    rate: rate.ok().flatten(),
                    chart: None,
                }
            }
            GemChartRequest::Chart { period, currency } => {
                self.remember_period(period);
                let rate = self.rate(currency.clone()).await;
                let chart = match &rate {
                    Ok(rate) => self.sync_charts(asset_id, period, currency, *rate).await,
                    Err(error) => Err(error.clone()),
                };
                GemChartResult {
                    request,
                    rate: rate.ok().flatten(),
                    state: GemLoadState::of(&chart),
                    chart: chart.ok(),
                }
            }
        }
    }
}

impl GemChartService {
    async fn rate(&self, currency: Currency) -> Result<Option<f64>, GemServiceError> {
        Ok(self.price.rate(currency).await?.map(|rate| rate.rate))
    }

    fn remember_period(&self, period: ChartPeriod) {
        if self.preferences.get_chart_period() != period {
            self.preferences.set_chart_period(period).ok();
        }
    }

    async fn sync_charts(&self, asset_id: AssetId, period: ChartPeriod, currency: Currency, rate: Option<f64>) -> Result<GemChart, GemServiceError> {
        let charts = self.api.client.get_charts(asset_id.clone(), period).await.map_err(GemApiError::from)?;
        if let Some(market) = charts.market {
            self.price.update_market(asset_id.clone(), market).await?;
        }
        let rate = rate.ok_or_else(|| GemServiceError::InvalidInput {
            msg: format!("unknown currency: {currency}"),
        })?;
        let latest = self.price.prices(vec![asset_id]).await?.into_iter().next();
        let values = rules::converted_values(charts.prices, rate);
        let base_value = rules::base_value(&values);
        let current = rules::current_value(&values, latest, Utc::now(), period, base_value);
        Ok(GemChart { values, base_value, current })
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, AssetMarket, Price, PriceProvider};

    use super::session::GemChartRate;
    use super::*;
    use crate::models::list::{GemListRow, GemListRowTitle};
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::services::price::testkit::MemoryPriceStore;
    use crate::services::price_alert::testkit::{MemoryPriceAlertStore, price_alert_service};
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;

    fn service(status: u16) -> GemChartService {
        GemChartService::new(
            Arc::new(GemApiClient::new(Arc::new(TestAlienProvider::with_json(status, "{}")))),
            Arc::new(GemPriceService::mock(Arc::new(MemoryPriceStore::with_rate(Currency::EUR, 0.5)))),
            Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))),
            Arc::new(GemExplorerService::mock()),
            price_alert_service(Arc::new(MemoryPriceAlertStore::default())),
        )
    }

    fn input(price: Option<f64>) -> GemChartInput {
        GemChartInput {
            asset: Asset::mock_ethereum_usdc(),
            price: price.map(|price| Price::new(price, 0.0, Utc::now(), PriceProvider::default())),
            market: Some(AssetMarket::mock()),
            price_alerts: vec![],
            links: vec![],
        }
    }

    fn titles(view: &GemChartView) -> Vec<GemListRowTitle> {
        view.sections
            .iter()
            .flat_map(|section| section.rows.iter())
            .filter_map(|row| match row {
                GemListRow::Ranked { title, .. } | GemListRow::Amount { title, .. } | GemListRow::Link { title, .. } => Some(*title),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_market_rows_convert_with_the_rate_the_session_holds_and_wait_for_it() {
        let service = service(200);
        let session = GemChartSession::new(ChartPeriod::Day, Currency::EUR);
        let pending = service.view_state(session.clone(), input(Some(1.0)));
        let rated = session.on_result(block_on(service.load(Asset::mock_ethereum_usdc().id, GemChartRequest::Rate { currency: Currency::EUR })));
        let shown = service.view_state(rated.clone(), input(Some(1.0)));

        assert_eq!(rated.rate, GemChartRate::Known { rate: 0.5 });
        assert!(!titles(&pending).contains(&GemListRowTitle::MarketCap), "a market the session cannot convert yet is not shown");
        assert!(titles(&pending).contains(&GemListRowTitle::SetPriceAlert), "the rows that need no rate do not wait for it");
        assert!(titles(&shown).contains(&GemListRowTitle::MarketCap));
        match shown
            .sections
            .iter()
            .flat_map(|section| section.rows.iter())
            .find(|row| matches!(row, GemListRow::Ranked { title: GemListRowTitle::MarketCap, .. }))
        {
            Some(GemListRow::Ranked { amount, .. }) => assert_eq!(amount.value, 50.0, "the market cap is converted with the session's rate"),
            other => panic!("expected a market cap row, got {other:?}"),
        }
    }

    #[test]
    fn test_a_chart_load_remembers_the_period_and_carries_the_rate_even_when_the_api_fails() {
        let service = service(500);
        let result = block_on(service.load(
            Asset::mock_ethereum_usdc().id,
            GemChartRequest::Chart {
                period: ChartPeriod::Month,
                currency: Currency::EUR,
            },
        ));

        assert_eq!(service.preferences.get_chart_period(), ChartPeriod::Month);
        assert_eq!(result.rate, Some(0.5));
        assert!(matches!(result.state, GemLoadState::Error { .. }));
        assert_eq!(result.chart, None);
        assert_eq!(service.new_session().period, ChartPeriod::Month, "the next chart opens on the period that was loaded last");
    }
}
