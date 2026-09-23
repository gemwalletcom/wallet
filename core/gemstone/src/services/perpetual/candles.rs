use chrono::{DateTime, TimeDelta, Utc};
use primitives::{ChartCandleUpdate, ChartPeriod, Perpetual};

use super::rules;
use crate::models::perpetual::GemChartCandleStick;
use crate::models::state::{GemLoad, GemLoadState};
use crate::services::chart::GemChartZoom;

const TRAILING_ROOM_FRACTION: f64 = 0.1;

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

#[derive(Debug, Default, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleViewport {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub interval_seconds: i64,
    pub candles: Vec<GemChartCandleStick>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleViewState {
    pub period: ChartPeriod,
    pub state: GemLoadState,
    pub viewport: GemCandleViewport,
    pub base: f64,
    pub is_refreshing: bool,
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
        let zoom = match self.candles.is_empty() {
            true => period_zoom(&shown.value, self.period),
            false => self.zoom,
        };
        Self {
            state: shown.state,
            candles: shown.value,
            is_refreshing: false,
            zoom,
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
            base: period_base(&self.candles, self.period),
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

fn period_zoom(candles: &[GemChartCandleStick], period: ChartPeriod) -> GemChartZoom {
    let (Some(first), Some(last)) = (candles.first(), candles.last()) else {
        return GemChartZoom::identity();
    };
    let periods = (last.date - first.date).num_minutes() as f64 / f64::from(period.minutes());
    GemChartZoom { scale: periods }.clamped(candles.len())
}

fn period_base(candles: &[GemChartCandleStick], period: ChartPeriod) -> f64 {
    let Some(last) = candles.last() else {
        return 0.0;
    };
    let start = last.date - TimeDelta::minutes(i64::from(period.minutes()));
    candles.iter().find(|candle| candle.date >= start).map_or(last.close, |candle| candle.close)
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
    let room = TimeDelta::milliseconds(((last.date - visible_start).num_milliseconds() as f64 * TRAILING_ROOM_FRACTION) as i64);
    GemCandleViewport {
        start: visible_start - interval / 2,
        end: last.date + room.max(interval / 2),
        interval_seconds: interval.num_seconds(),
        candles: candles[first_drawn..].to_vec(),
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
    fn test_a_first_load_opens_on_the_period_and_keeps_older_candles_for_zooming_out() {
        let session = candle_session(ChartPeriod::Hour).on_select_market(market("BTC"));
        let candles: Vec<GemChartCandleStick> = (0..240).map(|minute| GemChartCandleStick::mock(minute * 60, 100.0 + minute as f64)).collect();
        let shown = session.on_result(loaded(session.request().unwrap(), candles.clone()));
        let state = shown.view_state();

        assert_eq!(shown.zoom, GemChartZoom { scale: 239.0 / 60.0 });
        assert_eq!(state.viewport.candles, candles[179..], "the chart opens on the last hour, without a candle wholly off its edge");
        assert_eq!(state.base, candles[179].close, "the header change is over the period, not over the history kept for zooming out");
        assert_eq!(shown.candles, candles);
        assert_eq!(shown.on_zoom(0.1).zoom, GemChartZoom::identity(), "zooming out reaches the whole history");
        assert_eq!(
            shown.on_zoom(0.5).on_refresh().on_result(loaded(shown.request().unwrap(), candles)).zoom,
            GemChartZoom { scale: 239.0 / 120.0 },
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
