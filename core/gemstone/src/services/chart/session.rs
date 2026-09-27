use primitives::{AssetPrice, ChartPeriod, Currency};

use super::GemChart;
use super::model::GemChartData;
use super::rules;
use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
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

    fn phase(&self, price: Option<AssetPrice>) -> GemChartPhase {
        if self.is_loading {
            return GemChartPhase::Loading;
        }
        match (&self.chart, &self.error) {
            (Some(chart), _) => match rules::price_chart_data(rules::chart_with_price(chart.clone(), price, self.period), self.period, self.currency.clone()) {
                Some(data) => GemChartPhase::Data { data },
                None => GemChartPhase::NoData,
            },
            (None, Some(GemServiceError::Offline)) => GemChartPhase::Failed { error: GemServiceError::Offline },
            (None, Some(_)) => GemChartPhase::NoData,
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

    pub fn on_currency(&self, currency: Currency) -> Self {
        if currency == self.currency {
            return self.clone();
        }
        Self::new(self.period, currency)
    }

    pub fn view_state(&self, price: Option<AssetPrice>) -> GemChartViewState {
        GemChartViewState {
            period: self.period,
            phase: self.phase(price),
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

        assert_eq!(session.view_state(None).phase, GemChartPhase::NoData);
    }

    #[test]
    fn test_selecting_the_same_period_keeps_the_chart_it_already_loaded() {
        let loaded = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_loaded(GemChart::mock(vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]), ChartPeriod::Day);

        assert_eq!(loaded.on_select_period(ChartPeriod::Day), loaded, "reselecting a period is not a reload");
        assert_eq!(loaded.on_select_period(ChartPeriod::Week).view_state(None).phase, GemChartPhase::Loading);
    }

    #[test]
    fn test_a_refresh_keeps_the_visible_chart_and_marks_itself_refreshing() {
        let refreshing = GemChartSession::new(ChartPeriod::Day, Currency::USD)
            .on_loaded(GemChart::mock(vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]), ChartPeriod::Day)
            .on_refresh();
        let state = refreshing.view_state(None);

        assert!(state.is_refreshing);
        assert!(matches!(state.phase, GemChartPhase::Data { .. }), "the chart stays on screen while it refreshes");
        assert_eq!(GemChartSession::new(ChartPeriod::Day, Currency::USD).on_refresh().view_state(None).phase, GemChartPhase::Loading);
    }

    #[test]
    fn test_a_failure_after_a_load_keeps_the_chart_and_a_first_failure_reports_it() {
        let error = GemServiceError::Offline;
        let first = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_failed(error.clone(), ChartPeriod::Day);
        let after_load = GemChartSession::new(ChartPeriod::Day, Currency::USD)
            .on_loaded(GemChart::mock(vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)]), ChartPeriod::Day)
            .on_failed(error, ChartPeriod::Day);

        assert!(matches!(first.view_state(None).phase, GemChartPhase::Failed { .. }));
        assert!(matches!(after_load.view_state(None).phase, GemChartPhase::Data { .. }));
    }

    #[test]
    fn test_a_chart_the_server_cannot_answer_has_no_data_and_only_being_offline_is_an_error() {
        let missing = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_failed(GemServiceError::Api { msg: "Price not found".to_string() }, ChartPeriod::Day);
        let offline = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_failed(GemServiceError::Offline, ChartPeriod::Day);

        assert_eq!(missing.view_state(None).phase, GemChartPhase::NoData, "server text never reaches the chart");
        assert_eq!(offline.view_state(None).phase, GemChartPhase::Failed { error: GemServiceError::Offline });
    }

    #[test]
    fn test_a_newer_stored_price_moves_the_header_without_another_load() {
        let points = vec![ChartDateValue::mock(0, 1.0), ChartDateValue::mock(1_000, 2.0)];
        let loaded = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_loaded(GemChart::mock(points), ChartPeriod::Day);
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
        let header = |price| match loaded.view_state(price).phase {
            GemChartPhase::Data { data } => data.header.expect("a loaded chart has a header"),
            other => panic!("a loaded chart shows data, not {other:?}"),
        };

        assert_eq!(header(Some(newer)).value.value, 3.0, "the price the database holds is the one on top of the chart");
        assert_eq!(header(Some(older)).value.value, 2.0, "a price older than the last point does not move the header");
        assert_eq!(header(None).value.value, 2.0);
    }

    #[test]
    fn test_a_load_that_finished_after_the_period_changed_is_ignored() {
        let values = vec![ChartDateValue::mock(0, 0.0), ChartDateValue::mock(1, 1.0)];
        let selected_week = GemChartSession::new(ChartPeriod::Day, Currency::USD).on_select_period(ChartPeriod::Week);
        let error = GemServiceError::Core { msg: "offline".to_string() };

        assert_eq!(
            selected_week.on_loaded(GemChart::mock(values), ChartPeriod::Day).view_state(None).phase,
            GemChartPhase::Loading,
            "a chart for the period the user left behind never reaches the screen"
        );
        assert_eq!(selected_week.on_failed(error, ChartPeriod::Day).view_state(None).phase, GemChartPhase::Loading, "neither does its failure");
    }

    #[test]
    fn test_a_currency_change_drops_the_loaded_chart_and_keeps_the_period() {
        let loaded = GemChartSession::new(ChartPeriod::Week, Currency::USD).on_loaded(
            GemChart {
                values: Vec::new(),
                base_value: 0.0,
                current: None,
            },
            ChartPeriod::Week,
        );

        assert_eq!(loaded.on_currency(Currency::USD), loaded);
        let switched = loaded.on_currency(Currency::EUR);
        assert_eq!(switched, GemChartSession::new(ChartPeriod::Week, Currency::EUR));
        assert!(switched.is_loading);
    }
}
