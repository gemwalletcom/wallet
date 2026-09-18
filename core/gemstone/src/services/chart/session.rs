use primitives::{ChartPeriod, Currency};

use super::GemChart;
use super::model::GemChartData;
use super::rules;
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartPhase {
    Loading,
    Data { data: GemChartData },
    NoData,
    Failed { error: GemServiceError },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartViewState {
    pub period: ChartPeriod,
    pub phase: GemChartPhase,
    pub is_refreshing: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartSession {
    pub period: ChartPeriod,
    pub currency: Currency,
    pub chart: Option<GemChart>,
    pub error: Option<GemServiceError>,
    pub is_loading: bool,
    pub is_refreshing: bool,
}

impl GemChartSession {
    pub fn new(period: ChartPeriod, currency: Currency) -> Self {
        Self {
            period,
            currency,
            chart: None,
            error: None,
            is_loading: true,
            is_refreshing: false,
        }
    }

    fn phase(&self) -> GemChartPhase {
        if self.is_loading {
            return GemChartPhase::Loading;
        }
        match (&self.chart, &self.error) {
            (Some(chart), _) => match rules::price_chart_data(chart.clone(), self.currency.clone()) {
                Some(data) => GemChartPhase::Data { data },
                None => GemChartPhase::NoData,
            },
            (None, Some(error)) => GemChartPhase::Failed { error: error.clone() },
            (None, None) => GemChartPhase::NoData,
        }
    }
}

#[uniffi::export]
impl GemChartSession {
    pub fn on_select_period(&self, period: ChartPeriod) -> Self {
        if period == self.period {
            return self.clone();
        }
        Self::new(period, self.currency.clone())
    }

    pub fn view_state(&self) -> GemChartViewState {
        GemChartViewState {
            period: self.period,
            phase: self.phase(),
            is_refreshing: self.is_refreshing,
        }
    }

    pub fn on_refresh(&self) -> Self {
        Self {
            is_loading: self.chart.is_none(),
            is_refreshing: true,
            ..self.clone()
        }
    }

    pub fn on_loaded(&self, chart: GemChart, period: ChartPeriod) -> Self {
        if period != self.period {
            return self.clone();
        }
        Self {
            chart: Some(chart),
            error: None,
            is_loading: false,
            is_refreshing: false,
            ..self.clone()
        }
    }

    pub fn on_failed(&self, error: GemServiceError, period: ChartPeriod) -> Self {
        if period != self.period {
            return self.clone();
        }
        Self {
            error: Some(error),
            is_loading: false,
            is_refreshing: false,
            ..self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::ChartDateValue;

    #[test]
    fn test_a_chart_with_no_points_reads_as_no_data_not_as_a_failure() {
        let session = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_loaded(GemChart::mock(vec![]), ChartPeriod::Day);

        assert_eq!(session.view_state().phase, GemChartPhase::NoData);
    }

    #[test]
    fn test_selecting_the_same_period_keeps_the_chart_it_already_loaded() {
        let loaded =
            GemChartSession::new(ChartPeriod::Day, Currency::USD).on_loaded(GemChart::mock(vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]), ChartPeriod::Day);

        assert_eq!(loaded.on_select_period(ChartPeriod::Day), loaded, "reselecting a period is not a reload");
        assert_eq!(loaded.on_select_period(ChartPeriod::Week).view_state().phase, GemChartPhase::Loading);
    }

    #[test]
    fn test_a_refresh_keeps_the_visible_chart_and_marks_itself_refreshing() {
        let refreshing = GemChartSession::new(ChartPeriod::Day, Currency::USD)
            .on_loaded(GemChart::mock(vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]), ChartPeriod::Day)
            .on_refresh();
        let state = refreshing.view_state();

        assert!(state.is_refreshing);
        assert!(matches!(state.phase, GemChartPhase::Data { .. }), "the chart stays on screen while it refreshes");
        assert_eq!(
            GemChartSession::new(ChartPeriod::Day, Currency::USD).on_refresh().view_state().phase,
            GemChartPhase::Loading
        );
    }

    #[test]
    fn test_a_failure_after_a_load_keeps_the_chart_and_a_first_failure_reports_it() {
        let error = GemServiceError::Core { msg: "offline".to_string() };
        let first = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_failed(error.clone(), ChartPeriod::Day);
        let after_load = GemChartSession::new(ChartPeriod::Day, Currency::USD)
            .on_loaded(GemChart::mock(vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]), ChartPeriod::Day)
            .on_failed(error, ChartPeriod::Day);

        assert!(matches!(first.view_state().phase, GemChartPhase::Failed { .. }));
        assert!(matches!(after_load.view_state().phase, GemChartPhase::Data { .. }));
    }

    #[test]
    fn test_a_load_that_finished_after_the_period_changed_is_ignored() {
        let values = vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)];
        let selected_week = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_select_period(ChartPeriod::Week);
        let error = GemServiceError::Core { msg: "offline".to_string() };

        assert_eq!(
            selected_week.on_loaded(GemChart::mock(values), ChartPeriod::Day).view_state().phase,
            GemChartPhase::Loading,
            "a chart for the period the user left behind never reaches the screen"
        );
        assert_eq!(
            selected_week.on_failed(error, ChartPeriod::Day).view_state().phase,
            GemChartPhase::Loading,
            "neither does its failure"
        );
    }
}
