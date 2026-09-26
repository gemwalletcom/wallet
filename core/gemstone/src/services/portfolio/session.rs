use primitives::currency::Currency;
use primitives::{ChartPeriod, PortfolioChartType, PortfolioData, PortfolioType, WalletId};

use super::rules;
use crate::models::list::GemListRow;
use crate::models::state::{GemLoad, GemLoadState};
use crate::services::chart::GemChartData;
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioRequest {
    pub wallet_id: Option<WalletId>,
    pub portfolio_type: PortfolioType,
    pub period: ChartPeriod,
    pub currency: Currency,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioLoad {
    pub period: ChartPeriod,
    pub state: GemLoadState,
    pub data: Option<PortfolioData>,
}

impl GemPortfolioLoad {
    fn loading(period: ChartPeriod) -> Self {
        Self {
            period,
            state: GemLoadState::Loading,
            data: None,
        }
    }

    fn received(&self, period: ChartPeriod, state: GemLoadState, data: Option<PortfolioData>) -> Self {
        let shown = GemLoad {
            state: self.state.clone(),
            value: self.data.clone(),
        }
        .data(state.into_result(data));
        Self {
            period,
            state: shown.state,
            data: shown.value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioResult {
    pub request: GemPortfolioRequest,
    pub state: GemLoadState,
    pub data: Option<PortfolioData>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemPortfolioPhase {
    Loading,
    Data { chart: GemChartData },
    NoData,
    Failed { error: GemServiceError },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioViewState {
    pub portfolio_type: PortfolioType,
    pub period: ChartPeriod,
    pub chart_type: PortfolioChartType,
    pub periods: Vec<ChartPeriod>,
    pub currency: Currency,
    pub phase: GemPortfolioPhase,
    pub statistics: Vec<GemListRow>,
    pub shows_chart_type_picker: bool,
    pub is_refreshing: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioSession {
    pub wallet_id: Option<WalletId>,
    pub currency: Currency,
    pub portfolio_type: PortfolioType,
    pub period: ChartPeriod,
    pub chart_type: PortfolioChartType,
    pub periods: Vec<ChartPeriod>,
    pub wallet: GemPortfolioLoad,
    pub perpetuals: GemPortfolioLoad,
    pub is_refreshing: bool,
}

#[uniffi::export]
impl GemPortfolioSession {
    pub fn on_select_type(&self, portfolio_type: PortfolioType) -> Self {
        Self { portfolio_type, ..self.clone() }
    }

    pub fn on_select_chart_type(&self, chart_type: PortfolioChartType) -> Self {
        Self { chart_type, ..self.clone() }
    }

    pub fn on_select_period(&self, period: ChartPeriod) -> Self {
        if period == self.period {
            return self.clone();
        }
        Self {
            period,
            wallet: GemPortfolioLoad::loading(period),
            perpetuals: GemPortfolioLoad::loading(period),
            ..self.clone()
        }
    }

    pub fn on_select_wallet(&self, wallet_id: WalletId, currency: Currency) -> Self {
        if self.wallet_id.as_ref() == Some(&wallet_id) && currency == self.currency {
            return self.clone();
        }
        Self::new(Some(wallet_id), currency, self.portfolio_type, self.period, self.chart_type)
    }

    pub fn on_result(&self, result: GemPortfolioResult) -> Self {
        if !self.accepts(&result.request) {
            return self.clone();
        }
        let periods = result.data.as_ref().map(|data| data.available_periods.clone()).unwrap_or_default();
        let received = Self {
            is_refreshing: false,
            periods: match periods.is_empty() {
                true => self.periods.clone(),
                false => periods.clone(),
            },
            ..self.with_load(result.request.portfolio_type, self.load(result.request.portfolio_type).received(result.request.period, result.state, result.data))
        };
        match rules::fallback_period(result.request.period, &periods) {
            Some(period) => received.on_select_period(period),
            None => received,
        }
    }

    pub fn on_refresh(&self) -> Self {
        Self { is_refreshing: true, ..self.clone() }
    }

    pub fn request(&self) -> GemPortfolioRequest {
        GemPortfolioRequest {
            wallet_id: self.wallet_id.clone(),
            portfolio_type: self.portfolio_type,
            period: self.period,
            currency: self.currency.clone(),
        }
    }

    pub fn needs_load(&self) -> bool {
        if self.wallet_id.is_none() {
            return false;
        }
        let load = self.load(self.portfolio_type);
        self.is_refreshing || load.data.is_none() || load.period != self.period
    }

    pub fn view_state(&self) -> GemPortfolioViewState {
        let load = self.load(self.portfolio_type);
        let currency = rules::portfolio_currency(self.portfolio_type, self.currency.clone());
        GemPortfolioViewState {
            portfolio_type: self.portfolio_type,
            period: self.period,
            chart_type: self.chart_type,
            periods: self.periods.clone(),
            statistics: rules::statistic_rows(load.data.as_ref().map(|data| data.statistics.clone()).unwrap_or_default(), currency.clone()),
            phase: self.phase(load, currency.clone()),
            shows_chart_type_picker: self.portfolio_type == PortfolioType::Perpetuals,
            is_refreshing: self.is_refreshing,
            currency,
        }
    }
}

impl GemPortfolioSession {
    pub fn new(wallet_id: Option<WalletId>, currency: Currency, portfolio_type: PortfolioType, period: ChartPeriod, chart_type: PortfolioChartType) -> Self {
        Self {
            wallet_id,
            currency,
            portfolio_type,
            period,
            chart_type,
            periods: rules::wallet_periods(),
            wallet: GemPortfolioLoad::loading(period),
            perpetuals: GemPortfolioLoad::loading(period),
            is_refreshing: false,
        }
    }

    fn load(&self, portfolio_type: PortfolioType) -> &GemPortfolioLoad {
        match portfolio_type {
            PortfolioType::Wallet => &self.wallet,
            PortfolioType::Perpetuals => &self.perpetuals,
        }
    }

    fn with_load(&self, portfolio_type: PortfolioType, load: GemPortfolioLoad) -> Self {
        match portfolio_type {
            PortfolioType::Wallet => Self { wallet: load, ..self.clone() },
            PortfolioType::Perpetuals => Self { perpetuals: load, ..self.clone() },
        }
    }

    fn accepts(&self, request: &GemPortfolioRequest) -> bool {
        request.wallet_id == self.wallet_id && request.currency == self.currency && request.period == self.period
    }

    fn phase(&self, load: &GemPortfolioLoad, currency: Currency) -> GemPortfolioPhase {
        match (&load.state, &load.data) {
            (GemLoadState::Loading, _) => GemPortfolioPhase::Loading,
            (_, Some(data)) => match rules::portfolio_chart_data(data.clone(), self.portfolio_type, self.chart_type, self.period, currency) {
                Some(chart) => GemPortfolioPhase::Data { chart },
                None => GemPortfolioPhase::NoData,
            },
            (GemLoadState::Error { error: GemServiceError::Offline }, None) => GemPortfolioPhase::Failed { error: GemServiceError::Offline },
            (GemLoadState::Error { .. }, None) => GemPortfolioPhase::NoData,
            (GemLoadState::NoData | GemLoadState::Data, None) => GemPortfolioPhase::NoData,
        }
    }
}

#[uniffi::export]
pub fn portfolio_session(portfolio_type: PortfolioType) -> GemPortfolioSession {
    GemPortfolioSession::new(None, Currency::USD, portfolio_type, ChartPeriod::All, PortfolioChartType::Pnl)
}

#[cfg(test)]
mod tests {
    use primitives::{ChartDateValue, PortfolioChartData, PortfolioStatistic};

    use super::*;

    fn session() -> GemPortfolioSession {
        portfolio_session(PortfolioType::Wallet).on_select_wallet(WalletId::Multicoin("0x1".to_string()), Currency::USD)
    }

    fn loaded(request: GemPortfolioRequest, data: PortfolioData) -> GemPortfolioResult {
        GemPortfolioResult {
            request,
            state: GemLoadState::Data,
            data: Some(data),
        }
    }

    fn failed(request: GemPortfolioRequest, error: GemServiceError) -> GemPortfolioResult {
        GemPortfolioResult {
            request,
            state: GemLoadState::Error { error },
            data: None,
        }
    }

    fn data(periods: Vec<ChartPeriod>) -> PortfolioData {
        PortfolioData {
            charts: vec![PortfolioChartData {
                chart_type: PortfolioChartType::Value,
                values: vec![ChartDateValue::mock(0, 1.0), ChartDateValue::mock(1, 2.0)],
            }],
            statistics: vec![PortfolioStatistic::Volume { value: 10.0 }],
            available_periods: periods,
        }
    }

    #[test]
    fn test_a_result_for_another_selection_never_reaches_the_screen() {
        let selected = session().on_select_period(ChartPeriod::Week);
        let stale = GemPortfolioRequest {
            wallet_id: selected.wallet_id.clone(),
            portfolio_type: PortfolioType::Wallet,
            period: ChartPeriod::All,
            currency: Currency::USD,
        };

        assert_eq!(selected.on_result(loaded(stale.clone(), data(vec![ChartPeriod::Week]))), selected, "the period moved on before the answer arrived");
        assert_eq!(
            selected.on_result(failed(
                GemPortfolioRequest {
                    currency: Currency::EUR,
                    ..selected.request()
                },
                GemServiceError::Core { msg: "offline".to_string() }
            )),
            selected,
            "another currency's failure is not this one's"
        );
        assert_eq!(
            selected.on_result(loaded(
                GemPortfolioRequest {
                    wallet_id: Some(WalletId::Multicoin("0x2".to_string())),
                    ..selected.request()
                },
                data(vec![ChartPeriod::Week])
            )),
            selected,
            "another wallet's portfolio is not this wallet's"
        );
    }

    #[test]
    fn test_a_period_the_portfolio_does_not_offer_falls_back_to_the_first_one() {
        let session = session();
        let shown = session.on_result(loaded(session.request(), data(vec![ChartPeriod::Day, ChartPeriod::Week])));

        assert_eq!(shown.period, ChartPeriod::Day);
        assert!(shown.needs_load(), "the fallback period has nothing shown yet");
        assert_eq!(shown.view_state().periods, vec![ChartPeriod::Day, ChartPeriod::Week]);
    }

    #[test]
    fn test_each_type_keeps_its_own_load() {
        let session = session();
        let shown = session.on_result(loaded(session.request(), data(vec![ChartPeriod::All])));

        assert!(!shown.needs_load());
        let perpetuals = shown.on_select_type(PortfolioType::Perpetuals);
        assert!(perpetuals.needs_load(), "the perpetual portfolio has not shown yet");
        assert!(!perpetuals.on_select_type(PortfolioType::Wallet).needs_load(), "switching back shows what is already there");
        assert_eq!(perpetuals.view_state().currency, Currency::USD);
        assert!(perpetuals.view_state().shows_chart_type_picker);
    }

    #[test]
    fn test_a_failure_after_a_load_keeps_the_portfolio_on_screen() {
        let session = session();
        let error = GemServiceError::Offline;
        let shown = session.on_result(loaded(session.request(), data(vec![ChartPeriod::All])));

        assert!(matches!(session.on_result(failed(session.request(), error.clone())).view_state().phase, GemPortfolioPhase::Failed { .. }));
        assert!(!matches!(shown.on_result(failed(shown.request(), error)).view_state().phase, GemPortfolioPhase::Failed { .. }));
        assert_eq!(
            session.on_result(failed(session.request(), GemServiceError::Api { msg: "Not found".to_string() })).view_state().phase,
            GemPortfolioPhase::NoData,
            "server text never reaches the chart; only being offline is an error"
        );
    }

    #[test]
    fn test_a_refresh_asks_again_and_stops_when_the_answer_lands() {
        let session = session();
        let shown = session.on_result(loaded(session.request(), data(vec![ChartPeriod::All])));
        let refreshing = shown.on_refresh();

        assert!(refreshing.needs_load());
        assert!(refreshing.view_state().is_refreshing);
        assert!(!refreshing.on_result(loaded(refreshing.request(), data(vec![ChartPeriod::All]))).view_state().is_refreshing);
        assert!(
            !refreshing.on_result(failed(refreshing.request(), GemServiceError::Core { msg: "offline".to_string() })).view_state().is_refreshing,
            "a failed refresh stops the spinner too"
        );
    }

    #[test]
    fn test_a_wallet_change_starts_over_and_reselecting_the_same_wallet_does_not() {
        let session = session();
        let shown = session.on_result(loaded(session.request(), data(vec![ChartPeriod::All])));

        assert_eq!(shown.on_select_wallet(shown.wallet_id.clone().unwrap(), Currency::USD), shown);
        assert!(!portfolio_session(PortfolioType::Wallet).needs_load(), "there is nothing to load before a wallet is known");
        let switched = shown.on_select_wallet(WalletId::Multicoin("0x2".to_string()), Currency::USD);
        assert!(switched.needs_load());
        assert_eq!(switched.view_state().phase, GemPortfolioPhase::Loading);
    }
}
