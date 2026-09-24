use chrono::{DateTime, TimeDelta, Utc};
use primitives::{ChartCandleUpdate, ChartPeriod, Perpetual};

use super::rules;
use crate::models::perpetual::GemChartCandleStick;
use crate::models::state::{GemLoad, GemLoadState};
use crate::services::chart::{GemChartHeader, GemChartZoom, candlestick_header};

const TRAILING_ROOM_FRACTION: f64 = 0.1;
const MAX_TICKS: usize = 5;
const TICK_INSET_FRACTION: f64 = 0.1;
const TICK_YEAR_DAYS: i64 = 360;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemCandleRequest {
    pub symbol: String,
    pub period: ChartPeriod,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleResult {
    pub request: GemCandleRequest,
    pub state: GemLoadState,
    pub candles: Vec<GemChartCandleStick>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCandleTickFormat {
    #[default]
    Time,
    TimeOrDay,
    Day,
    MonthYear,
}

#[derive(Debug, Default, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleViewport {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub interval_seconds: i64,
    pub candles: Vec<GemChartCandleStick>,
    pub ticks: Vec<DateTime<Utc>>,
    pub tick_format: GemCandleTickFormat,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleViewState {
    pub period: ChartPeriod,
    pub state: GemLoadState,
    pub viewport: GemCandleViewport,
    pub base: f64,
    pub is_refreshing: bool,
}

#[uniffi::export]
impl GemCandleViewState {
    pub fn header_at(&self, value: f64) -> GemChartHeader {
        candlestick_header(self.base, value)
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleSession {
    pub symbol: Option<String>,
    pub period: ChartPeriod,
    pub state: GemLoadState,
    pub candles: Vec<GemChartCandleStick>,
    pub is_refreshing: bool,
    pub zoom: GemChartZoom,
}

#[uniffi::export]
impl GemCandleSession {
    pub fn on_select_market(&self, perpetual: Perpetual) -> Self {
        let symbol = rules::symbol(&perpetual);
        if self.symbol.as_deref() == Some(symbol.as_str()) {
            return self.clone();
        }
        Self {
            symbol: Some(symbol),
            ..Self::empty(self.period)
        }
    }

    pub fn on_select_period(&self, period: ChartPeriod) -> Self {
        if period == self.period {
            return self.clone();
        }
        Self {
            symbol: self.symbol.clone(),
            ..Self::empty(period)
        }
    }

    pub fn on_refresh(&self) -> Self {
        match self.symbol {
            Some(_) => Self { is_refreshing: true, ..self.clone() },
            None => self.clone(),
        }
    }

    pub fn on_result(&self, result: GemCandleResult) -> Self {
        if self.request().as_ref() != Some(&result.request) {
            return self.clone();
        }
        let shown = GemLoad {
            state: self.state.clone(),
            value: self.candles.clone(),
        }
        .data(result.state.into_result(result.candles));
        Self {
            state: shown.state,
            candles: shown.value,
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn on_zoom(&self, magnification: f64) -> Self {
        Self {
            zoom: self.zoom.magnified(magnification, self.candles.len()),
            ..self.clone()
        }
    }

    pub fn on_candle_update(&self, update: ChartCandleUpdate) -> Self {
        let Some(symbol) = &self.symbol else {
            return self.clone();
        };
        match rules::merged_candles(self.candles.clone(), update, symbol, &self.period) {
            Some(candles) => Self {
                state: GemLoadState::Data,
                candles,
                is_refreshing: false,
                ..self.clone()
            },
            None => self.clone(),
        }
    }

    pub fn request(&self) -> Option<GemCandleRequest> {
        self.symbol.clone().map(|symbol| GemCandleRequest { symbol, period: self.period })
    }

    pub fn needs_candles(&self) -> bool {
        self.symbol.is_some() && (self.is_refreshing || matches!(self.state, GemLoadState::Loading))
    }

    pub fn view_state(&self) -> GemCandleViewState {
        GemCandleViewState {
            period: self.period,
            state: match (&self.state, self.candles.is_empty()) {
                (GemLoadState::Data, true) => GemLoadState::NoData,
                (state, _) => state.clone(),
            },
            viewport: viewport(&self.candles, self.zoom),
            base: self.candles.first().map_or(0.0, |candle| candle.close),
            is_refreshing: self.is_refreshing,
        }
    }
}

impl GemCandleSession {
    fn empty(period: ChartPeriod) -> Self {
        Self {
            symbol: None,
            period,
            state: GemLoadState::Loading,
            candles: Vec::new(),
            is_refreshing: false,
            zoom: GemChartZoom::identity(),
        }
    }
}

fn viewport(candles: &[GemChartCandleStick], zoom: GemChartZoom) -> GemCandleViewport {
    let (Some(first), Some(last)) = (candles.first(), candles.last()) else {
        return GemCandleViewport::default();
    };
    let interval = candles
        .iter()
        .zip(&candles[1..])
        .map(|(previous, next)| next.date - previous.date)
        .filter(|gap| *gap > TimeDelta::zero())
        .min()
        .unwrap_or_default();
    let visible_start = zoom.clamped(candles.len()).visible_start(first.date, last.date);
    let first_drawn = candles.partition_point(|candle| candle.date <= visible_start - interval).min(candles.len() - 1);
    let room = fraction_of(last.date - visible_start, TRAILING_ROOM_FRACTION);
    let start = visible_start - interval / 2;
    let end = last.date + room.max(interval / 2);
    let drawn = &candles[first_drawn..];
    let inset = start + fraction_of(end - start, TICK_INSET_FRACTION);
    let labelled = &drawn[drawn.partition_point(|candle| candle.date < inset)..];
    let per_tick = candles_per_tick(labelled.len(), interval);
    GemCandleViewport {
        start,
        end,
        interval_seconds: interval.num_seconds(),
        candles: drawn.to_vec(),
        ticks: labelled.iter().rev().step_by(per_tick).rev().map(|candle| candle.date).collect(),
        tick_format: tick_format(end - start, interval * per_tick as i32, last.date - drawn[0].date),
    }
}

fn fraction_of(span: TimeDelta, fraction: f64) -> TimeDelta {
    TimeDelta::milliseconds((span.num_milliseconds() as f64 * fraction) as i64)
}

fn candles_per_tick(count: usize, interval: TimeDelta) -> usize {
    let gaps = count.saturating_sub(1) as f64;
    let rough = gaps / (MAX_TICKS - 1) as f64;
    let fitting = rough.floor().max(1.0);
    let candles = match gaps / fitting < MAX_TICKS as f64 {
        true => fitting,
        false => rough.ceil(),
    } as usize;
    let day = TimeDelta::days(1);
    match interval > TimeDelta::zero() && interval < day && interval * candles as i32 >= day {
        true => {
            let per_day = (day.num_seconds() as f64 / interval.num_seconds() as f64).round() as usize;
            candles.div_ceil(per_day) * per_day
        }
        false => candles,
    }
}

fn tick_format(span: TimeDelta, step: TimeDelta, covered: TimeDelta) -> GemCandleTickFormat {
    let day = TimeDelta::days(1);
    if span >= TimeDelta::days(TICK_YEAR_DAYS) {
        GemCandleTickFormat::MonthYear
    } else if step >= day {
        GemCandleTickFormat::Day
    } else if covered <= day {
        GemCandleTickFormat::Time
    } else {
        GemCandleTickFormat::TimeOrDay
    }
}

#[uniffi::export]
pub fn candle_session(period: ChartPeriod) -> GemCandleSession {
    GemCandleSession::empty(period)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::error::GemServiceError;

    fn candle(timestamp: i64) -> GemChartCandleStick {
        GemChartCandleStick {
            date: chrono::DateTime::from_timestamp(timestamp, 0).unwrap(),
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
            volume: 10.0,
        }
    }

    fn offline() -> GemServiceError {
        GemServiceError::Gateway { msg: "offline".to_string() }
    }

    fn market(name: &str) -> Perpetual {
        Perpetual { name: name.to_string(), ..Perpetual::mock() }
    }

    fn session() -> GemCandleSession {
        candle_session(ChartPeriod::Day).on_select_market(market("BTC"))
    }

    fn loaded(request: GemCandleRequest, candles: Vec<GemChartCandleStick>) -> GemCandleResult {
        GemCandleResult { request, state: GemLoadState::Data, candles }
    }

    fn failed(request: GemCandleRequest) -> GemCandleResult {
        GemCandleResult {
            request,
            state: GemLoadState::Error { error: offline() },
            candles: Vec::new(),
        }
    }

    #[test]
    fn test_a_market_with_no_candles_reads_as_empty_rather_than_a_chart_of_nothing() {
        let session = session();

        assert_eq!(session.view_state().state, GemLoadState::Loading);
        assert_eq!(session.on_result(loaded(session.request().unwrap(), vec![])).view_state().state, GemLoadState::NoData);
        assert_eq!(session.on_result(loaded(session.request().unwrap(), vec![candle(1)])).view_state().state, GemLoadState::Data);
    }

    #[test]
    fn test_a_failed_refresh_keeps_the_candles_on_screen() {
        let shown = session().on_result(loaded(session().request().unwrap(), vec![candle(1)]));

        let kept = shown.on_refresh().on_result(failed(shown.request().unwrap()));

        assert_eq!(kept.view_state().state, GemLoadState::Data);
        assert_eq!(kept.view_state().viewport.candles, vec![candle(1)]);
        assert!(!kept.view_state().is_refreshing, "a failed refresh stops the spinner too");
        assert!(matches!(session().on_result(failed(session().request().unwrap())).view_state().state, GemLoadState::Error { .. }));
    }

    #[test]
    fn test_candles_for_a_period_the_screen_moved_past_never_reach_it() {
        let session = session();
        let stale = session.request().unwrap();
        let selected = session.on_select_period(ChartPeriod::Week);

        assert_eq!(selected.on_result(loaded(stale.clone(), vec![candle(1)])), selected, "the period moved on before the answer arrived");
        assert_eq!(
            selected.on_result(loaded(GemCandleRequest { symbol: "ETH".to_string(), ..stale }, vec![candle(1)])),
            selected,
            "another market's candles are not this one's"
        );
        assert!(selected.needs_candles());
        assert!(!candle_session(ChartPeriod::Day).needs_candles(), "there is nothing to ask for before a market is known");
        assert!(
            !candle_session(ChartPeriod::Day).on_refresh().view_state().is_refreshing,
            "a market the screen does not have yet cannot be refreshed, so the spinner never starts"
        );
    }

    #[test]
    fn test_viewport() {
        let candles: Vec<GemChartCandleStick> = (0..20).map(|minute| GemChartCandleStick::mock(minute * 60, 100.0 + minute as f64)).collect();
        let whole = viewport(&candles, GemChartZoom::identity());
        let zoomed = viewport(&candles, GemChartZoom { scale: 2.0 });
        let gapped = viewport(&[GemChartCandleStick::mock(0, 1.0), GemChartCandleStick::mock(60, 1.0), GemChartCandleStick::mock(600, 1.0)], GemChartZoom::identity());

        assert_eq!(
            (whole.start, whole.end),
            (DateTime::from_timestamp(-30, 0).unwrap(), DateTime::from_timestamp(1254, 0).unwrap()),
            "the window covers whole candles and keeps a share of itself empty after the newest"
        );
        assert_eq!(whole.candles, candles);
        assert_eq!(whole.interval_seconds, 60);
        assert_eq!((zoomed.start, zoomed.end), (DateTime::from_timestamp(540, 0).unwrap(), DateTime::from_timestamp(1197, 0).unwrap()));
        assert_eq!(zoomed.candles, candles[9..], "the candle straddling the left edge stays partially visible");
        assert_eq!(gapped.interval_seconds, 60, "a gap in trading does not widen the candles");
        assert_eq!(viewport(&candles[..1], GemChartZoom::identity()).candles, candles[..1], "a lone candle is still drawn");
        assert_eq!(viewport(&[], GemChartZoom::identity()), GemCandleViewport::default());
    }

    #[test]
    fn test_ticks() {
        let series = |from: &str, seconds: i64, count: i64| -> Vec<GemChartCandleStick> {
            let first = DateTime::parse_from_rfc3339(from).unwrap().timestamp();
            (0..count).map(|index| GemChartCandleStick::mock(first + index * seconds, 1.0)).collect()
        };
        let at = |dates: &[&str]| -> Vec<DateTime<Utc>> { dates.iter().map(|date| DateTime::parse_from_rfc3339(date).unwrap().to_utc()).collect() };
        let axis = |candles: Vec<GemChartCandleStick>| {
            let viewport = viewport(&candles, GemChartZoom::identity());
            (viewport.ticks, viewport.tick_format)
        };
        let monthly: Vec<GemChartCandleStick> = (0..14)
            .map(|month| GemChartCandleStick::mock(DateTime::parse_from_rfc3339("2025-08-07T00:00:00Z").unwrap().checked_add_months(chrono::Months::new(month)).unwrap().timestamp(), 1.0))
            .collect();

        assert_eq!(
            axis(series("2026-09-23T15:08:00Z", 60, 61)),
            (
                at(&["2026-09-23T15:16:00Z", "2026-09-23T15:29:00Z", "2026-09-23T15:42:00Z", "2026-09-23T15:55:00Z", "2026-09-23T16:08:00Z"]),
                GemCandleTickFormat::Time
            ),
            "an hour is labelled on its candles counted back from the newest"
        );
        assert_eq!(
            axis(series("2026-09-22T16:30:00Z", 1800, 48)),
            (
                at(&["2026-09-22T20:00:00Z", "2026-09-23T01:00:00Z", "2026-09-23T06:00:00Z", "2026-09-23T11:00:00Z", "2026-09-23T16:00:00Z"]),
                GemCandleTickFormat::Time
            ),
            "candles that fit in a day are labelled with times only"
        );
        assert_eq!(
            axis(series("2026-09-22T04:00:00Z", 4 * 3600, 9)),
            (at(&["2026-09-22T12:00:00Z", "2026-09-22T20:00:00Z", "2026-09-23T04:00:00Z", "2026-09-23T12:00:00Z"]), GemCandleTickFormat::TimeOrDay)
        );
        assert_eq!(
            axis(series("2026-09-16T16:00:00Z", 4 * 3600, 43)),
            (at(&["2026-09-17T16:00:00Z", "2026-09-19T16:00:00Z", "2026-09-21T16:00:00Z", "2026-09-23T16:00:00Z"]), GemCandleTickFormat::Day),
            "a step of a day or more lands on whole days of candles"
        );
        assert_eq!(
            axis(monthly),
            (
                at(&["2025-09-07T00:00:00Z", "2025-12-07T00:00:00Z", "2026-03-07T00:00:00Z", "2026-06-07T00:00:00Z", "2026-09-07T00:00:00Z"]),
                GemCandleTickFormat::MonthYear
            ),
            "a chart over a year names the month and year"
        );
        for (seconds, count) in [(60, 1), (60, 60), (1800, 48), (4 * 3600, 42), (12 * 3600, 60), (7 * 86400, 52), (30 * 86400, 40)] {
            let candles = series("2026-01-05T00:00:00Z", seconds, count);
            let (ticks, _) = axis(candles.clone());
            assert!(ticks.iter().all(|tick| candles.iter().any(|candle| candle.date == *tick)), "{count} candles of {seconds} s");
            assert_eq!(ticks.last(), candles.last().map(|candle| &candle.date), "{count} candles of {seconds} s");
            assert!(ticks.len() <= MAX_TICKS, "{count} candles of {seconds} s");
        }
    }

    #[test]
    fn test_on_zoom() {
        let candles: Vec<GemChartCandleStick> = (0..40).map(|minute| GemChartCandleStick::mock(minute * 60, 100.0)).collect();
        let zoomed = session().on_result(loaded(session().request().unwrap(), candles.clone())).on_zoom(3.0);

        assert_eq!(zoomed.zoom, GemChartZoom { scale: 3.0 });
        assert_eq!(zoomed.on_zoom(3.0).zoom, GemChartZoom { scale: 5.0 }, "the session clamps against the candles it holds");
        assert_eq!(
            zoomed
                .on_candle_update(ChartCandleUpdate {
                    coin: "BTC".to_string(),
                    interval: rules::candle_interval(&ChartPeriod::Day).to_string(),
                    candle: GemChartCandleStick::mock(40 * 60, 100.0),
                })
                .zoom,
            zoomed.zoom,
            "a streamed candle keeps the zoom"
        );
        assert_eq!(zoomed.on_select_period(ChartPeriod::Week).zoom, GemChartZoom::identity(), "a new period starts unzoomed");
        assert_eq!(zoomed.on_select_market(market("ETH")).zoom, GemChartZoom::identity());
    }

    #[test]
    fn test_on_result() {
        let candles: Vec<GemChartCandleStick> = (0..60).map(|minute| GemChartCandleStick::mock(minute * 60, 100.0 + minute as f64)).collect();
        let shown = session().on_result(loaded(session().request().unwrap(), candles.clone()));
        let state = shown.view_state();

        assert_eq!(state.viewport.candles, candles, "a first load shows the whole period");
        assert_eq!(state.header_at(candles[59].close), candlestick_header(candles[0].close, candles[59].close), "the header change is over the whole period");
        assert_eq!(
            shown.on_zoom(2.0).on_refresh().on_result(loaded(shown.request().unwrap(), candles)).zoom,
            GemChartZoom { scale: 2.0 },
            "a refresh keeps the zoom the user chose"
        );
        assert_eq!(candle_session(ChartPeriod::Hour).view_state().base, 0.0);
    }

    #[test]
    fn test_on_candle_update() {
        let shown = session().on_result(loaded(session().request().unwrap(), vec![candle(1)]));
        let update = ChartCandleUpdate {
            coin: "BTC".to_string(),
            interval: rules::candle_interval(&ChartPeriod::Day).to_string(),
            candle: candle(2),
        };
        let updated = shown.on_candle_update(update.clone());
        let week = shown.on_select_period(ChartPeriod::Week).on_result(loaded(
            GemCandleRequest {
                symbol: "BTC".to_string(),
                period: ChartPeriod::Week,
            },
            vec![candle(1)],
        ));
        let eth = shown.on_select_market(market("ETH")).on_result(loaded(
            GemCandleRequest {
                symbol: "ETH".to_string(),
                period: ChartPeriod::Day,
            },
            vec![candle(1)],
        ));

        assert_eq!(updated.candles, vec![candle(2)]);
        assert_eq!(updated.view_state().state, GemLoadState::Data);
        assert!(!updated.needs_candles(), "a streamed candle replaces what is shown without asking again");
        assert_eq!(week.on_candle_update(update.clone()), week, "a candle streamed for the period the screen left never reaches the new one");
        assert_eq!(eth.on_candle_update(update.clone()), eth, "another market's candle is not this chart's");
        assert_eq!(session().on_candle_update(update), session(), "a candle streamed before the first load is not a chart");
    }
}
