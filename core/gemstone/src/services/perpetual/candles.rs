use chrono::TimeDelta;
use primitives::{Asset, ChartCandleUpdate, ChartPeriod, Perpetual, PerpetualPosition};

use super::model::GemCandleChart;
use super::{chart, rules};
use crate::models::perpetual::GemChartCandleStick;
use crate::models::state::{GemLoad, GemLoadState};
use crate::perpetual::GemPerpetual;
use crate::services::chart::GemChartZoom;

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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCandleViewState {
    pub period: ChartPeriod,
    pub state: GemLoadState,
    pub candles: Vec<GemChartCandleStick>,
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
        Self {
            state: shown.state,
            candles: shown.value,
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn on_zoom(&self, magnification: f64, anchor: f64) -> Self {
        Self {
            zoom: self.zoom.magnified(magnification, anchor, self.candles.len()),
            ..self.clone()
        }
    }

    pub fn on_pan(&self, fraction: f64) -> Self {
        Self {
            zoom: self.zoom.panned(fraction, self.candles.len()),
            ..self.clone()
        }
    }

    pub fn on_candle_update(&self, update: ChartCandleUpdate) -> Self {
        match self.symbol.as_ref().and_then(|symbol| rules::merged_candles(&self.candles, update, symbol, &self.period)) {
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
        let needs_candles = self.is_refreshing || matches!(self.state, GemLoadState::Loading);
        self.symbol.clone().filter(|_| needs_candles).map(|symbol| GemCandleRequest { symbol, period: self.period })
    }

    pub fn chart(&self, asset: Asset, position: Option<PerpetualPosition>, utc_offset_seconds: i32) -> Option<GemCandleChart> {
        let provider = rules::provider(asset.chain())?;
        let price_decimals = GemPerpetual::new(provider).price_decimals(self.candles.last()?.close, asset.decimals);
        chart::candle_chart(&self.candles, self.period, price_decimals, position.as_ref(), self.zoom, TimeDelta::seconds(i64::from(utc_offset_seconds)))
    }

    pub fn view_state(&self) -> GemCandleViewState {
        GemCandleViewState {
            period: self.period,
            state: match (&self.state, self.candles.is_empty()) {
                (GemLoadState::Data, true) => GemLoadState::NoData,
                (state, _) => state.clone(),
            },
            candles: self.candles.clone(),
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
            zoom: GemChartZoom::default(),
        }
    }
}

#[uniffi::export]
pub fn candle_session(period: ChartPeriod) -> GemCandleSession {
    GemCandleSession::empty(period)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatted_number::GemNumberDisplay;
    use crate::precision::GemPrecision;
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

        let refreshing = shown.on_refresh();
        let kept = refreshing.on_result(failed(refreshing.request().unwrap()));

        assert_eq!(kept.view_state().state, GemLoadState::Data);
        assert_eq!(kept.view_state().candles, vec![candle(1)]);
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
        assert!(selected.request().is_some());
        assert!(candle_session(ChartPeriod::Day).request().is_none(), "there is nothing to ask for before a market is known");
        assert!(
            !candle_session(ChartPeriod::Day).on_refresh().view_state().is_refreshing,
            "a market the screen does not have yet cannot be refreshed, so the spinner never starts"
        );
    }

    #[test]
    fn test_on_zoom() {
        let candles = GemChartCandleStick::mock_series(chrono::DateTime::UNIX_EPOCH, TimeDelta::minutes(1), 70);
        let shown = session().on_result(loaded(session().request().unwrap(), candles.clone()));
        let zoomed = shown.on_zoom(100.0, 1.0);
        let refreshing = zoomed.on_refresh();

        assert_eq!(zoomed.zoom, GemChartZoom { scale: 5.0, offset: 0.0 }, "the session clamps against the candles it holds");
        assert_eq!(zoomed.chart(Asset::mock_perpetual(), None, 0).map(|chart| chart.candles.len()), Some(15), "the chart draws the zoomed window");
        assert_eq!(refreshing.on_result(loaded(refreshing.request().unwrap(), candles)).zoom, zoomed.zoom, "a refresh keeps the zoom");
        assert_eq!(zoomed.on_select_period(ChartPeriod::Week).zoom, GemChartZoom::default(), "a new period starts unzoomed");
    }

    #[test]
    fn test_chart() {
        let shown = |close: f64| session().on_result(loaded(session().request().unwrap(), vec![GemChartCandleStick { close, ..candle(1) }]));
        let chart = |close: f64, decimals: u32| shown(close).chart(Asset { decimals, ..Asset::mock_perpetual() }, None, 0).unwrap();
        let xrp = chart(1.4251, 0);

        assert_eq!(
            (xrp.price_decimals, xrp.header.value.display),
            (
                4,
                GemNumberDisplay::Number {
                    precision: GemPrecision::Fraction { min: 4, max: 4 }
                }
            ),
            "XRP reads 1.4251 as on Hyperliquid, not $1.43"
        );
        assert_eq!(chart(4512.3, 4).price_decimals, 1, "five significant figures");
        assert_eq!(chart(0.5, 4).price_decimals, 2, "no more places than the market's size decimals leave");
        assert_eq!(shown(1.4251).chart(Asset::mock(), None, 0), None, "an asset with no perpetual market has no candle chart");
    }

    #[test]
    fn test_on_pan() {
        let shown = session().on_result(loaded(session().request().unwrap(), GemChartCandleStick::mock_series(chrono::DateTime::UNIX_EPOCH, TimeDelta::minutes(1), 70)));

        assert_eq!(shown.on_zoom(4.0, 1.0).on_pan(0.4).zoom, GemChartZoom { scale: 4.0, offset: 0.1 });
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

        assert_eq!((updated.request(), updated.candles), (None, vec![candle(2)]), "a streamed candle replaces what is shown without asking again");
        assert_eq!(session().on_candle_update(update), session(), "a candle streamed before the first load is not a chart");
    }
}
