use primitives::{ChartPeriod, Perpetual};

use super::rules;
use crate::models::perpetual::GemChartCandleStick;
use crate::models::state::{GemLoad, GemLoadState};

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

    pub fn on_candles(&self, candles: Vec<GemChartCandleStick>) -> Self {
        Self {
            state: GemLoadState::Data,
            candles,
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn request(&self) -> Option<GemCandleRequest> {
        let needs_candles = self.is_refreshing || matches!(self.state, GemLoadState::Loading);
        self.symbol.clone().filter(|_| needs_candles).map(|symbol| GemCandleRequest { symbol, period: self.period })
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
    fn test_a_streamed_candle_replaces_what_is_shown_without_asking_again() {
        let shown = session().on_result(loaded(session().request().unwrap(), vec![candle(1)]));

        let merged = shown.on_candles(vec![candle(1), candle(2)]);

        assert_eq!(merged.view_state().candles.len(), 2);
        assert!(merged.request().is_none());
        assert!(shown.request().is_none(), "shown candles are not asked for again until a refresh");
    }
}
