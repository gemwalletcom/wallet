use chrono::TimeDelta;
use primitives::{Asset, AssetLink, AssetMarket, AssetPrice, ChartCandleStick, ChartPeriod, Currency, Price, PriceAlert};

use super::GemChart;
use super::model::{GemChartData, GemChartStyle};
use super::rules;
use super::zoom::GemChartZoom;
use crate::models::list::GemListSection;
use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;
use crate::services::perpetual::chart::candle_chart;
use crate::services::perpetual::model::GemCandleChart;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemChartPhase {
    Loading,
    Data { data: GemChartData },
    Candles { chart: GemCandleChart },
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
    pub utc_offset_seconds: i32,
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
    pub style_switch: Option<GemChartStyle>,
    pub periods: Vec<ChartPeriod>,
    pub sections: Vec<GemListSection>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartRequest {
    Rate { currency: Currency },
    Chart { period: ChartPeriod, currency: Currency },
    Candles { period: ChartPeriod, currency: Currency },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartResult {
    pub request: GemChartRequest,
    pub rate: Option<f64>,
    pub state: GemLoadState,
    pub chart: Option<GemChart>,
    pub candles: Vec<ChartCandleStick>,
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
    pub style: GemChartStyle,
    pub has_candles: bool,
    pub candles: Vec<ChartCandleStick>,
    pub candles_error: Option<GemServiceError>,
    pub needs_candles: bool,
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
            style: GemChartStyle::Line,
            has_candles: false,
            candles: Vec::new(),
            candles_error: None,
            needs_candles: true,
        }
    }

    pub fn with_style(self, style: GemChartStyle) -> Self {
        Self { style, ..self }
    }

    pub fn rate(&self) -> Option<f64> {
        match self.rate {
            GemChartRate::Known { rate } => Some(rate),
            GemChartRate::Pending | GemChartRate::Unknown => None,
        }
    }

    pub fn phase(&self, price: Option<AssetPrice>, utc_offset_seconds: i32) -> GemChartPhase {
        if self.is_loading {
            return GemChartPhase::Loading;
        }
        if self.shows_candles() {
            return self.candles_phase(price, utc_offset_seconds);
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

    pub fn style_switch(&self) -> Option<GemChartStyle> {
        self.has_candles.then_some(match self.style {
            GemChartStyle::Line => GemChartStyle::Candles,
            GemChartStyle::Candles => GemChartStyle::Line,
        })
    }

    pub fn periods(&self) -> Vec<ChartPeriod> {
        [ChartPeriod::Hour, ChartPeriod::Day, ChartPeriod::Week, ChartPeriod::Month, ChartPeriod::Year, ChartPeriod::All]
            .into_iter()
            .filter(|period| !self.shows_candles() || rules::has_candle_period(*period))
            .collect()
    }

    pub(super) fn on_select_style(&self, style: GemChartStyle) -> Self {
        if style == self.style || (style == GemChartStyle::Candles && !self.has_candles) {
            return self.clone();
        }
        let selected = Self {
            style,
            zoom: GemChartZoom::default(),
            ..self.clone()
        };
        if selected.shows_candles() { selected.with_candle_period() } else { selected }
    }

    fn shows_candles(&self) -> bool {
        self.style == GemChartStyle::Candles && self.has_candles
    }

    fn with_candle_period(self) -> Self {
        if rules::has_candle_period(self.period) {
            return self;
        }
        Self {
            rate: self.rate,
            style: self.style,
            has_candles: self.has_candles,
            ..Self::new(rules::DEFAULT_CANDLE_PERIOD, self.currency)
        }
    }

    fn candles_phase(&self, price: Option<AssetPrice>, utc_offset_seconds: i32) -> GemChartPhase {
        let candles = rules::candles_with_price(&self.candles, price);
        let chart = candles.last().and_then(|last| {
            candle_chart(
                &candles,
                self.period,
                rules::candle_price_decimals(last.close),
                self.currency.clone(),
                None,
                &[],
                self.zoom,
                TimeDelta::seconds(i64::from(utc_offset_seconds)),
            )
        });
        match (chart, &self.candles_error) {
            (Some(chart), _) => GemChartPhase::Candles { chart },
            (None, _) if self.needs_candles => GemChartPhase::Loading,
            (None, Some(GemServiceError::Offline)) => GemChartPhase::Failed { error: GemServiceError::Offline },
            (None, _) => GemChartPhase::NoData,
        }
    }

    fn points(&self) -> usize {
        match self.shows_candles() {
            true => self.candles.len(),
            false => self.chart.as_ref().map_or(0, |chart| chart.values.len()),
        }
    }

    fn accepts(&self, request: &GemChartRequest) -> bool {
        match request {
            GemChartRequest::Rate { currency } => currency == &self.currency,
            GemChartRequest::Chart { period, currency } | GemChartRequest::Candles { period, currency } => period == &self.period && currency == &self.currency,
        }
    }

    fn with_rate(&self, rate: Option<f64>) -> GemChartRate {
        match rate {
            Some(rate) => GemChartRate::Known { rate },
            None => self.rate.clone(),
        }
    }

    fn loaded(&self, chart: GemChart, rate: Option<f64>) -> Self {
        let loaded = Self {
            rate: self.with_rate(rate),
            has_candles: chart.has_candles,
            chart: Some(chart),
            error: None,
            is_loading: false,
            is_refreshing: false,
            ..self.clone()
        };
        if loaded.shows_candles() { loaded.with_candle_period() } else { loaded }
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
            style: self.style,
            has_candles: self.has_candles,
            ..Self::new(period, self.currency.clone())
        }
    }

    pub fn on_currency(&self, currency: Currency) -> Self {
        if currency == self.currency {
            return self.clone();
        }
        Self {
            style: self.style,
            has_candles: self.has_candles,
            ..Self::new(self.period, currency)
        }
    }

    pub fn request(&self) -> Option<GemChartRequest> {
        if self.rate == GemChartRate::Pending {
            return Some(GemChartRequest::Rate { currency: self.currency.clone() });
        }
        if self.is_loading || self.is_refreshing {
            return Some(GemChartRequest::Chart {
                period: self.period,
                currency: self.currency.clone(),
            });
        }
        (self.shows_candles() && self.needs_candles).then(|| GemChartRequest::Candles {
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
            GemChartRequest::Candles { .. } => match result.state.into_result(result.candles) {
                Ok(candles) => Self {
                    rate: self.with_rate(result.rate),
                    candles,
                    candles_error: None,
                    needs_candles: false,
                    ..self.clone()
                },
                Err(error) => Self {
                    rate: self.with_rate(result.rate),
                    candles_error: Some(error),
                    needs_candles: false,
                    ..self.clone()
                },
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
            needs_candles: true,
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
            candles: vec![],
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
            candles: vec![],
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
            candles: vec![],
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
        assert_eq!(session.phase(None, 0), GemChartPhase::Loading);
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
            candles: vec![],
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

        assert_eq!(session.phase(None, 0), GemChartPhase::NoData);
    }

    #[test]
    fn test_selecting_the_same_period_keeps_the_chart_it_already_loaded() {
        let loaded = loaded(rated(ChartPeriod::Day, Currency::USD), two_points());

        assert_eq!(loaded.on_select_period(ChartPeriod::Day), loaded, "reselecting a period is not a reload");
        assert_eq!(loaded.on_select_period(ChartPeriod::Week).phase(None, 0), GemChartPhase::Loading);
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
        assert!(matches!(refreshing.phase(None, 0), GemChartPhase::Data { .. }), "the chart stays on screen while it refreshes");
        assert_eq!(rated(ChartPeriod::Day, Currency::USD).on_refresh().phase(None, 0), GemChartPhase::Loading);
    }

    #[test]
    fn test_a_failure_after_a_load_keeps_the_chart_and_a_first_failure_reports_it() {
        let session = rated(ChartPeriod::Day, Currency::USD);
        let first = session.on_result(failed_result(&session, GemServiceError::Offline));
        let shown = loaded(session, two_points());
        let after_load = shown.on_result(failed_result(&shown, GemServiceError::Offline));

        assert!(matches!(first.phase(None, 0), GemChartPhase::Failed { .. }));
        assert!(matches!(after_load.phase(None, 0), GemChartPhase::Data { .. }));
    }

    #[test]
    fn test_a_chart_the_server_cannot_answer_has_no_data_and_only_being_offline_is_an_error() {
        let session = rated(ChartPeriod::Day, Currency::USD);
        let missing = session.on_result(failed_result(&session, GemServiceError::Api { msg: "Price not found".to_string() }));
        let offline = session.on_result(failed_result(&session, GemServiceError::Offline));

        assert_eq!(missing.phase(None, 0), GemChartPhase::NoData, "server text never reaches the chart");
        assert_eq!(offline.phase(None, 0), GemChartPhase::Failed { error: GemServiceError::Offline });
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
        let header = |price| match loaded.phase(price, 0) {
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
            selected_week.on_result(chart_result(&day, GemChart::mock(two_points()))).phase(None, 0),
            GemChartPhase::Loading,
            "a chart for the period the user left behind never reaches the screen"
        );
        assert_eq!(selected_week.on_result(failed_result(&day, error)).phase(None, 0), GemChartPhase::Loading, "neither does its failure");
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
                candles: vec![],
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

    fn candle(hour: u32, close: f64) -> ChartCandleStick {
        ChartCandleStick {
            date: chrono::DateTime::from_timestamp(i64::from(hour) * 3_600, 0).unwrap(),
            open: close,
            high: close,
            low: close,
            close,
            volume: 0.0,
        }
    }

    fn with_candles(session: GemChartSession) -> GemChartSession {
        session.on_result(chart_result(
            &session,
            GemChart {
                has_candles: true,
                ..GemChart::mock(two_points())
            },
        ))
    }

    fn candles_result(session: &GemChartSession, candles: Vec<ChartCandleStick>) -> GemChartResult {
        GemChartResult {
            request: session.request().unwrap(),
            rate: Some(1.0),
            state: GemLoadState::Data,
            chart: None,
            candles,
        }
    }

    #[test]
    fn test_an_asset_without_candles_offers_no_switch_and_stays_on_the_line() {
        let session = loaded(rated(ChartPeriod::Day, Currency::USD), two_points());

        assert_eq!(session.style_switch(), None);
        assert_eq!(session.on_select_style(GemChartStyle::Candles), session);
        assert_eq!(session.periods().len(), 6);
    }

    #[test]
    fn test_a_remembered_candle_style_shows_the_line_for_an_asset_without_candles() {
        let session = loaded(rated(ChartPeriod::Day, Currency::USD).with_style(GemChartStyle::Candles), two_points());

        assert_eq!(session.request(), None);
        assert!(matches!(session.phase(None, 0), GemChartPhase::Data { .. }));
    }

    #[test]
    fn test_switching_to_candles_loads_them_and_back_keeps_the_line() {
        let line = with_candles(rated(ChartPeriod::Day, Currency::USD));
        assert_eq!(line.style_switch(), Some(GemChartStyle::Candles));

        let waiting = line.on_select_style(GemChartStyle::Candles);
        assert_eq!(
            waiting.request(),
            Some(GemChartRequest::Candles {
                period: ChartPeriod::Day,
                currency: Currency::USD
            })
        );
        assert_eq!(waiting.phase(None, 0), GemChartPhase::Loading);
        assert_eq!(waiting.style_switch(), Some(GemChartStyle::Line));

        let shown = waiting.on_result(candles_result(&waiting, vec![candle(0, 1.0), candle(1, 2.0)]));
        assert_eq!(shown.request(), None);
        assert!(matches!(shown.phase(None, 0), GemChartPhase::Candles { .. }));

        let back = shown.on_select_style(GemChartStyle::Line);
        assert_eq!(back.request(), None);
        assert!(matches!(back.phase(None, 0), GemChartPhase::Data { .. }));
    }

    #[test]
    fn test_candles_have_no_hour_period() {
        let hour = with_candles(rated(ChartPeriod::Hour, Currency::USD));

        let candles = hour.on_select_style(GemChartStyle::Candles);

        assert_eq!(candles.period, ChartPeriod::Day);
        assert!(!candles.periods().contains(&ChartPeriod::Hour));
        assert_eq!(
            candles.request(),
            Some(GemChartRequest::Chart {
                period: ChartPeriod::Day,
                currency: Currency::USD
            }),
            "the line for the new period loads too, so switching back shows the right one"
        );
        assert_eq!(hour.on_select_style(GemChartStyle::Line).period, ChartPeriod::Hour);
    }

    #[test]
    fn test_a_remembered_candle_style_opens_on_hour_at_a_period_with_candles() {
        let session = with_candles(rated(ChartPeriod::Hour, Currency::USD).with_style(GemChartStyle::Candles));

        assert_eq!(session.period, ChartPeriod::Day);
        assert!(session.has_candles);
    }

    #[test]
    fn test_a_period_change_keeps_the_style_and_loads_the_line_then_the_candles() {
        let waiting = with_candles(rated(ChartPeriod::Day, Currency::USD)).on_select_style(GemChartStyle::Candles);
        let shown = waiting.on_result(candles_result(&waiting, vec![candle(0, 1.0)]));

        let week = shown.on_select_period(ChartPeriod::Week);

        assert_eq!(week.style, GemChartStyle::Candles);
        assert!(week.candles.is_empty());
        assert!(matches!(week.request(), Some(GemChartRequest::Chart { period: ChartPeriod::Week, .. })));
        let line_loaded = with_candles(week);
        assert!(matches!(line_loaded.request(), Some(GemChartRequest::Candles { period: ChartPeriod::Week, .. })));
    }

    #[test]
    fn test_a_late_candle_result_for_another_period_is_ignored() {
        let waiting = with_candles(rated(ChartPeriod::Day, Currency::USD)).on_select_style(GemChartStyle::Candles);
        let stale = candles_result(&waiting, vec![candle(0, 1.0)]);

        let week = with_candles(waiting.on_select_period(ChartPeriod::Week));

        assert_eq!(week.on_result(stale), week);
    }

    #[test]
    fn test_a_refresh_keeps_the_candles_on_screen_and_a_failed_one_keeps_them() {
        let waiting = with_candles(rated(ChartPeriod::Day, Currency::USD)).on_select_style(GemChartStyle::Candles);
        let shown = waiting.on_result(candles_result(&waiting, vec![candle(0, 1.0), candle(1, 2.0)]));

        let refreshed = with_candles(shown.on_refresh());
        assert!(matches!(refreshed.request(), Some(GemChartRequest::Candles { .. })));
        assert!(matches!(refreshed.phase(None, 0), GemChartPhase::Candles { .. }));

        let failed = refreshed.on_result(GemChartResult {
            state: GemLoadState::Error { error: GemServiceError::Offline },
            ..candles_result(&refreshed, vec![])
        });
        assert_eq!(failed.request(), None);
        assert!(matches!(failed.phase(None, 0), GemChartPhase::Candles { .. }));
    }

    #[test]
    fn test_candles_that_cannot_load_show_no_data_or_offline() {
        let waiting = with_candles(rated(ChartPeriod::Day, Currency::USD)).on_select_style(GemChartStyle::Candles);

        assert_eq!(waiting.on_result(candles_result(&waiting, vec![])).phase(None, 0), GemChartPhase::NoData);
        let offline = waiting.on_result(GemChartResult {
            state: GemLoadState::Error { error: GemServiceError::Offline },
            ..candles_result(&waiting, vec![])
        });
        assert_eq!(offline.phase(None, 0), GemChartPhase::Failed { error: GemServiceError::Offline });
    }

    #[test]
    fn test_zoom_counts_candles_while_they_show() {
        let waiting = with_candles(rated(ChartPeriod::Day, Currency::USD)).on_select_style(GemChartStyle::Candles);
        let shown = waiting.on_result(candles_result(&waiting, (0..70).map(|hour| candle(hour, 1.0)).collect()));

        assert_eq!(shown.on_zoom(100.0, 1.0).zoom.scale, 5.0);
        assert_eq!(shown.on_zoom(100.0, 1.0).on_select_style(GemChartStyle::Line).zoom, GemChartZoom::default());
    }
}
