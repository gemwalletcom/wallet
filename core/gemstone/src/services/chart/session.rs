use primitives::{Asset, AssetLink, AssetMarket, AssetPrice, ChartPeriod, Currency, Price, PriceAlert};

use super::GemChart;
use super::model::GemChartData;
use super::rules;
use super::zoom::GemChartZoom;
use crate::models::list::GemListSection;
use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemChartPhase {
    Loading,
    Data { data: GemChartData },
    NoData,
    Failed { error: GemServiceError },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemChartInput {
    pub asset: Asset,
    pub price: Option<Price>,
    pub market: Option<AssetMarket>,
    pub price_alerts: Vec<PriceAlert>,
    pub links: Vec<AssetLink>,
}

impl GemChartInput {
    pub fn asset_price(&self) -> Option<AssetPrice> {
        self.price.as_ref().map(|price| AssetPrice::new(self.asset.id.clone(), price.price, price.price_change_percentage_24h, price.updated_at))
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartView {
    pub period: ChartPeriod,
    pub phase: GemChartPhase,
    pub is_refreshing: bool,
    pub sections: Vec<GemListSection>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartRequest {
    Rate { currency: Currency },
    Chart { period: ChartPeriod, currency: Currency },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartResult {
    pub request: GemChartRequest,
    pub rate: Option<f64>,
    pub state: GemLoadState,
    pub chart: Option<GemChart>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartRate {
    Pending,
    Unknown,
    Known { rate: f64 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartSession {
    pub period: ChartPeriod,
    pub currency: Currency,
    pub rate: GemChartRate,
    pub chart: Option<GemChart>,
    pub error: Option<GemServiceError>,
    pub is_loading: bool,
    pub is_refreshing: bool,
    pub zoom: GemChartZoom,
}

impl GemChartSession {
    pub fn new(period: ChartPeriod, currency: Currency) -> Self {
        Self {
            period,
            currency,
            rate: GemChartRate::Pending,
            chart: None,
            error: None,
            is_loading: true,
            is_refreshing: false,
            zoom: GemChartZoom::default(),
        }
    }

    pub fn rate(&self) -> Option<f64> {
        match self.rate {
            GemChartRate::Known { rate } => Some(rate),
            GemChartRate::Pending | GemChartRate::Unknown => None,
        }
    }

    pub fn phase(&self, price: Option<AssetPrice>) -> GemChartPhase {
        if self.is_loading {
            return GemChartPhase::Loading;
        }
        match (&self.chart, &self.error) {
            (Some(chart), _) => match rules::price_chart_data(rules::chart_with_price(chart.clone(), price, self.period), self.period, self.currency.clone()) {
                Some(data) => GemChartPhase::Data { data: rules::zoomed_chart(data, self.zoom) },
                None => GemChartPhase::NoData,
            },
            (None, Some(GemServiceError::Offline)) => GemChartPhase::Failed { error: GemServiceError::Offline },
            (None, Some(_)) => GemChartPhase::NoData,
            (None, None) => GemChartPhase::NoData,
        }
    }

    fn points(&self) -> usize {
        self.chart.as_ref().map_or(0, |chart| chart.values.len())
    }

    fn accepts(&self, request: &GemChartRequest) -> bool {
        match request {
            GemChartRequest::Rate { currency } => currency == &self.currency,
            GemChartRequest::Chart { period, currency } => period == &self.period && currency == &self.currency,
        }
    }

    fn with_rate(&self, rate: Option<f64>) -> GemChartRate {
        match rate {
            Some(rate) => GemChartRate::Known { rate },
            None => self.rate.clone(),
        }
    }

    fn loaded(&self, chart: GemChart, rate: Option<f64>) -> Self {
        Self {
            rate: self.with_rate(rate),
            chart: Some(chart),
            error: None,
            is_loading: false,
            is_refreshing: false,
            ..self.clone()
        }
    }

    fn failed(&self, error: GemServiceError, rate: Option<f64>) -> Self {
        Self {
            rate: self.with_rate(rate),
            error: Some(error),
            is_loading: false,
            is_refreshing: false,
            ..self.clone()
        }
    }
}

#[uniffi::export]
impl GemChartSession {
    pub fn on_select_period(&self, period: ChartPeriod) -> Self {
        if period == self.period {
            return self.clone();
        }
        Self {
            rate: self.rate.clone(),
            ..Self::new(period, self.currency.clone())
        }
    }

    pub fn on_currency(&self, currency: Currency) -> Self {
        if currency == self.currency {
            return self.clone();
        }
        Self::new(self.period, currency)
    }

    pub fn request(&self) -> Option<GemChartRequest> {
        if self.rate == GemChartRate::Pending {
            return Some(GemChartRequest::Rate { currency: self.currency.clone() });
        }
        (self.is_loading || self.is_refreshing).then(|| GemChartRequest::Chart {
            period: self.period,
            currency: self.currency.clone(),
        })
    }

    pub fn on_result(&self, result: GemChartResult) -> Self {
        if !self.accepts(&result.request) {
            return self.clone();
        }
        match result.request {
            GemChartRequest::Rate { .. } => Self {
                rate: result.rate.map_or(GemChartRate::Unknown, |rate| GemChartRate::Known { rate }),
                ..self.clone()
            },
            GemChartRequest::Chart { .. } => match result.state.into_result(result.chart) {
                Ok(Some(chart)) => self.loaded(chart, result.rate),
                Ok(None) => self.clone(),
                Err(error) => self.failed(error, result.rate),
            },
        }
    }

    pub fn on_zoom(&self, magnification: f64, anchor: f64) -> Self {
        Self {
            zoom: self.zoom.magnified(magnification, anchor, self.points()),
            ..self.clone()
        }
    }

    pub fn on_pan(&self, fraction: f64) -> Self {
        Self {
            zoom: self.zoom.panned(fraction, self.points()),
            ..self.clone()
        }
    }

    pub fn on_refresh(&self) -> Self {
        Self {
            is_loading: self.chart.is_none(),
            is_refreshing: true,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::ChartDateValue;

    fn rated(period: ChartPeriod, currency: Currency) -> GemChartSession {
        GemChartSession::new(period, currency.clone()).on_result(GemChartResult {
            request: GemChartRequest::Rate { currency },
            rate: Some(1.0),
            state: GemLoadState::Data,
            chart: None,
        })
    }

    fn chart_result(session: &GemChartSession, chart: GemChart) -> GemChartResult {
        GemChartResult {
            request: GemChartRequest::Chart {
                period: session.period,
                currency: session.currency.clone(),
            },
            rate: Some(1.0),
            state: GemLoadState::Data,
            chart: Some(chart),
        }
    }

    fn failed_result(session: &GemChartSession, error: GemServiceError) -> GemChartResult {
        GemChartResult {
            request: GemChartRequest::Chart {
                period: session.period,
                currency: session.currency.clone(),
            },
            rate: None,
            state: GemLoadState::Error { error },
            chart: None,
        }
    }

    fn loaded(session: GemChartSession, values: Vec<ChartDateValue>) -> GemChartSession {
        let result = chart_result(&session, GemChart::mock(values));
        session.on_result(result)
    }

    fn two_points() -> Vec<ChartDateValue> {
        vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]
    }

    #[test]
    fn test_a_new_session_asks_for_the_rate_before_the_chart() {
        let session = GemChartSession::new(ChartPeriod::Day, Currency::EUR);

        assert_eq!(session.request(), Some(GemChartRequest::Rate { currency: Currency::EUR }));
        assert_eq!(session.phase(None), GemChartPhase::Loading);
        assert_eq!(
            rated(ChartPeriod::Day, Currency::EUR).request(),
            Some(GemChartRequest::Chart {
                period: ChartPeriod::Day,
                currency: Currency::EUR
            })
        );
    }

    #[test]
    fn test_a_rate_the_store_does_not_have_still_lets_the_chart_load() {
        let unknown = GemChartSession::new(ChartPeriod::Day, Currency::EUR).on_result(GemChartResult {
            request: GemChartRequest::Rate { currency: Currency::EUR },
            rate: None,
            state: GemLoadState::Error { error: GemServiceError::Offline },
            chart: None,
        });

        assert_eq!(unknown.rate, GemChartRate::Unknown);
        assert_eq!(unknown.rate(), None);
        assert!(matches!(unknown.request(), Some(GemChartRequest::Chart { .. })), "a missing rate is answered once, not asked forever");
    }

    #[test]
    fn test_a_chart_load_refreshes_the_rate_it_fetched() {
        let session = rated(ChartPeriod::Day, Currency::EUR);
        let result = GemChartResult {
            rate: Some(0.9),
            ..chart_result(&session, GemChart::mock(two_points()))
        };

        assert_eq!(session.on_result(result).rate(), Some(0.9));
        assert_eq!(session.on_result(failed_result(&session, GemServiceError::Offline)).rate(), Some(1.0), "a failed load keeps the rate it had");
    }

    #[test]
    fn test_a_chart_with_no_points_reads_as_no_data_not_as_a_failure() {
        let session = loaded(rated(ChartPeriod::Day, Currency::USD), vec![]);

        assert_eq!(session.phase(None), GemChartPhase::NoData);
    }

    #[test]
    fn test_selecting_the_same_period_keeps_the_chart_it_already_loaded() {
        let loaded = loaded(rated(ChartPeriod::Day, Currency::USD), two_points());

        assert_eq!(loaded.on_select_period(ChartPeriod::Day), loaded, "reselecting a period is not a reload");
        assert_eq!(loaded.on_select_period(ChartPeriod::Week).phase(None), GemChartPhase::Loading);
    }

    #[test]
    fn test_a_new_period_keeps_the_rate_and_asks_for_the_chart_only() {
        let week = loaded(rated(ChartPeriod::Day, Currency::USD), two_points()).on_select_period(ChartPeriod::Week);

        assert_eq!(week.rate(), Some(1.0));
        assert_eq!(
            week.request(),
            Some(GemChartRequest::Chart {
                period: ChartPeriod::Week,
                currency: Currency::USD
            })
        );
    }

    #[test]
    fn test_a_refresh_keeps_the_visible_chart_and_marks_itself_refreshing() {
        let refreshing = loaded(rated(ChartPeriod::Day, Currency::USD), two_points()).on_refresh();

        assert!(refreshing.is_refreshing);
        assert!(matches!(refreshing.phase(None), GemChartPhase::Data { .. }), "the chart stays on screen while it refreshes");
        assert_eq!(rated(ChartPeriod::Day, Currency::USD).on_refresh().phase(None), GemChartPhase::Loading);
    }

    #[test]
    fn test_a_failure_after_a_load_keeps_the_chart_and_a_first_failure_reports_it() {
        let session = rated(ChartPeriod::Day, Currency::USD);
        let first = session.on_result(failed_result(&session, GemServiceError::Offline));
        let shown = loaded(session, two_points());
        let after_load = shown.on_result(failed_result(&shown, GemServiceError::Offline));

        assert!(matches!(first.phase(None), GemChartPhase::Failed { .. }));
        assert!(matches!(after_load.phase(None), GemChartPhase::Data { .. }));
    }

    #[test]
    fn test_a_chart_the_server_cannot_answer_has_no_data_and_only_being_offline_is_an_error() {
        let session = rated(ChartPeriod::Day, Currency::USD);
        let missing = session.on_result(failed_result(&session, GemServiceError::Api { msg: "Price not found".to_string() }));
        let offline = session.on_result(failed_result(&session, GemServiceError::Offline));

        assert_eq!(missing.phase(None), GemChartPhase::NoData, "server text never reaches the chart");
        assert_eq!(offline.phase(None), GemChartPhase::Failed { error: GemServiceError::Offline });
    }

    #[test]
    fn test_a_newer_stored_price_moves_the_header_without_another_load() {
        let points = vec![ChartDateValue::mock(0, 1.0), ChartDateValue::mock(1_000, 2.0)];
        let loaded = loaded(rated(ChartPeriod::Day, Currency::USD), points);
        let newer = AssetPrice {
            asset_id: primitives::AssetId::from_chain(primitives::Chain::Bitcoin),
            price: 3.0,
            price_change_percentage_24h: 50.0,
            updated_at: chrono::DateTime::from_timestamp(2_000, 0).expect("timestamp"),
        };
        let older = AssetPrice {
            updated_at: chrono::DateTime::from_timestamp(0, 0).expect("timestamp"),
            ..newer.clone()
        };
        let header = |price| match loaded.phase(price) {
            GemChartPhase::Data { data } => data.header.expect("a loaded chart has a header"),
            other => panic!("a loaded chart shows data, not {other:?}"),
        };

        assert_eq!(header(Some(newer)).value.value, 3.0, "the price the database holds is the one on top of the chart");
        assert_eq!(header(Some(older)).value.value, 2.0, "a price older than the last point does not move the header");
        assert_eq!(header(None).value.value, 2.0);
    }

    #[test]
    fn test_on_zoom() {
        let values = ChartDateValue::mock_series(140);
        let shown = loaded(rated(ChartPeriod::Day, Currency::USD), values.clone());
        let zoomed = shown.on_zoom(100.0, 1.0);

        assert_eq!(zoomed.zoom, GemChartZoom { scale: 10.0, offset: 0.0 }, "the session clamps against the points it holds");
        assert_eq!(rated(ChartPeriod::Day, Currency::USD).on_zoom(4.0, 1.0).zoom, GemChartZoom::default(), "nothing loaded, nothing to zoom");
        let refreshed = zoomed.on_refresh();
        assert_eq!(refreshed.on_result(chart_result(&refreshed, GemChart::mock(values))).zoom, zoomed.zoom, "a refresh keeps the zoom");
        assert_eq!(zoomed.on_select_period(ChartPeriod::Week).zoom, GemChartZoom::default(), "a new period starts unzoomed");
    }

    #[test]
    fn test_request() {
        let session = rated(ChartPeriod::Day, Currency::USD);
        let shown = loaded(session.clone(), ChartDateValue::mock_series(140));

        assert_eq!(
            session.request(),
            Some(GemChartRequest::Chart {
                period: ChartPeriod::Day,
                currency: Currency::USD
            })
        );
        assert_eq!(shown.request(), None, "a shown chart is not asked for again");
        assert_eq!(shown.on_refresh().request(), session.request(), "a refresh asks again");
        assert_eq!(shown.on_refresh().on_zoom(4.0, 1.0).request(), session.request(), "a pinch keeps the request that is in flight");
    }

    #[test]
    fn test_on_pan() {
        let zoomed = loaded(rated(ChartPeriod::Day, Currency::USD), ChartDateValue::mock_series(140)).on_zoom(4.0, 1.0);

        assert_eq!(zoomed.on_pan(0.4).zoom, GemChartZoom { scale: 4.0, offset: 0.1 });
    }

    #[test]
    fn test_a_load_that_finished_after_the_period_changed_is_ignored() {
        let day = rated(ChartPeriod::Day, Currency::USD);
        let selected_week = day.on_select_period(ChartPeriod::Week);
        let error = GemServiceError::Core { msg: "offline".to_string() };

        assert_eq!(
            selected_week.on_result(chart_result(&day, GemChart::mock(two_points()))).phase(None),
            GemChartPhase::Loading,
            "a chart for the period the user left behind never reaches the screen"
        );
        assert_eq!(selected_week.on_result(failed_result(&day, error)).phase(None), GemChartPhase::Loading, "neither does its failure");
    }

    #[test]
    fn test_a_load_that_finished_after_the_currency_changed_is_ignored() {
        let usd = rated(ChartPeriod::Day, Currency::USD);
        let eur = usd.on_currency(Currency::EUR);

        assert_eq!(eur.request(), Some(GemChartRequest::Rate { currency: Currency::EUR }), "a new currency needs its own rate first");
        assert_eq!(eur.on_result(chart_result(&usd, GemChart::mock(two_points()))), eur, "a chart priced in the old currency never reaches the screen");
        assert_eq!(
            eur.on_result(GemChartResult {
                request: GemChartRequest::Rate { currency: Currency::USD },
                rate: Some(1.0),
                state: GemLoadState::Data,
                chart: None,
            }),
            eur,
            "neither does the old currency's rate"
        );
    }

    #[test]
    fn test_a_currency_change_drops_the_loaded_chart_and_keeps_the_period() {
        let shown = loaded(rated(ChartPeriod::Week, Currency::USD), vec![]);

        assert_eq!(shown.on_currency(Currency::USD), shown);
        let switched = shown.on_currency(Currency::EUR);
        assert_eq!(switched, GemChartSession::new(ChartPeriod::Week, Currency::EUR));
        assert!(switched.is_loading);
    }
}
