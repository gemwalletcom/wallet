use primitives::{
    ChartPeriod, ChartValuePercentage, PerpetualPortfolio, PerpetualPortfolioTimeframeData, PortfolioAssets, PortfolioChartData, PortfolioChartType, PortfolioData,
    PortfolioMarginUsage, PortfolioStatistic,
};

use super::model::GemPortfolioValues;
use crate::services::chart::rules::converted_values;

pub fn converted_portfolio(portfolio: PortfolioAssets, rate: f64) -> GemPortfolioValues {
    GemPortfolioValues {
        values: converted_values(portfolio.values, rate),
        all_time_high: portfolio.all_time_high.map(|value| converted_percentage(value, rate)),
        all_time_low: portfolio.all_time_low.map(|value| converted_percentage(value, rate)),
    }
}

fn converted_percentage(value: ChartValuePercentage, rate: f64) -> ChartValuePercentage {
    ChartValuePercentage {
        date: value.date,
        value: (value.value as f64 * rate) as f32,
        percentage: value.percentage,
    }
}

pub fn wallet_periods() -> Vec<ChartPeriod> {
    vec![ChartPeriod::Day, ChartPeriod::Week, ChartPeriod::Month, ChartPeriod::Year, ChartPeriod::All]
}

pub fn wallet_portfolio_data(values: GemPortfolioValues) -> PortfolioData {
    let statistics = [
        values.all_time_high.map(PortfolioStatistic::AllTimeHigh),
        values.all_time_low.map(PortfolioStatistic::AllTimeLow),
    ]
    .into_iter()
    .flatten()
    .collect();

    PortfolioData {
        charts: vec![PortfolioChartData {
            chart_type: PortfolioChartType::Value,
            values: values.values,
        }],
        statistics,
        available_periods: wallet_periods(),
    }
}

pub fn perpetual_portfolio_data(portfolio: PerpetualPortfolio, period: ChartPeriod) -> PortfolioData {
    let timeframe = timeframe_data(&portfolio, period);
    let charts = vec![
        PortfolioChartData {
            chart_type: PortfolioChartType::Pnl,
            values: timeframe.map(|data| data.pnl_history.clone()).unwrap_or_default(),
        },
        PortfolioChartData {
            chart_type: PortfolioChartType::Value,
            values: timeframe
                .map(|data| data.account_value_history.iter().skip_while(|value| value.value == 0.0).cloned().collect())
                .unwrap_or_default(),
        },
    ];

    let mut statistics = Vec::new();
    if let Some(summary) = &portfolio.account_summary {
        statistics.push(PortfolioStatistic::UnrealizedPnl(summary.unrealized_pnl));
        statistics.push(PortfolioStatistic::AccountLeverage(summary.account_leverage));
        statistics.push(PortfolioStatistic::MarginUsage(PortfolioMarginUsage {
            account_value: summary.account_value,
            usage: summary.margin_usage,
        }));
    }
    if let Some(all_time) = &portfolio.all_time {
        if let Some(last) = all_time.pnl_history.last() {
            statistics.push(PortfolioStatistic::AllTimePnl(last.value));
        }
        statistics.push(PortfolioStatistic::Volume(all_time.volume));
    }

    PortfolioData {
        charts,
        statistics,
        available_periods: perpetual_periods(&portfolio),
    }
}

fn perpetual_periods(portfolio: &PerpetualPortfolio) -> Vec<ChartPeriod> {
    [
        portfolio.day.as_ref().map(|_| ChartPeriod::Day),
        portfolio.week.as_ref().map(|_| ChartPeriod::Week),
        portfolio.month.as_ref().map(|_| ChartPeriod::Month),
        portfolio.all_time.as_ref().map(|_| ChartPeriod::Year),
        portfolio.all_time.as_ref().map(|_| ChartPeriod::All),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn timeframe_data(portfolio: &PerpetualPortfolio, period: ChartPeriod) -> Option<&PerpetualPortfolioTimeframeData> {
    match period {
        ChartPeriod::Hour | ChartPeriod::Day => portfolio.day.as_ref(),
        ChartPeriod::Week => portfolio.week.as_ref(),
        ChartPeriod::Month => portfolio.month.as_ref(),
        ChartPeriod::Year | ChartPeriod::All => portfolio.all_time.as_ref(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use primitives::{ChartDateValue, ChartValue, PerpetualAccountSummary};

    use super::*;

    #[test]
    fn test_converted_portfolio_applies_rate_to_values_and_extremes() {
        let now = Utc::now();
        let portfolio = PortfolioAssets {
            total_value: 10.0,
            values: vec![ChartValue { timestamp: 2, value: 2.0 }, ChartValue { timestamp: 1, value: 1.0 }],
            all_time_high: Some(ChartValuePercentage {
                date: now,
                value: 4.0,
                percentage: 10.0,
            }),
            all_time_low: None,
            allocation: vec![],
        };
        let converted = converted_portfolio(portfolio, 2.0);
        assert_eq!(converted.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![2.0, 4.0]);
        let high = converted.all_time_high.unwrap();
        assert_eq!(high.value, 8.0);
        assert_eq!(high.percentage, 10.0);
        assert!(converted.all_time_low.is_none());
    }

    #[test]
    fn test_perpetual_portfolio_data_keeps_the_statistics_the_portfolio_has() {
        let mut portfolio = PerpetualPortfolio::mock();
        portfolio.all_time = Some(PerpetualPortfolioTimeframeData::mock());
        portfolio.account_summary = Some(PerpetualAccountSummary {
            account_value: 100.0,
            account_leverage: 2.0,
            margin_usage: 0.5,
            unrealized_pnl: 7.0,
        });

        let data = perpetual_portfolio_data(portfolio.clone(), ChartPeriod::Day);

        assert_eq!(
            data.charts.iter().map(|chart| chart.chart_type).collect::<Vec<_>>(),
            vec![PortfolioChartType::Pnl, PortfolioChartType::Value]
        );
        assert_eq!(
            data.statistics,
            vec![
                PortfolioStatistic::UnrealizedPnl(7.0),
                PortfolioStatistic::AccountLeverage(2.0),
                PortfolioStatistic::MarginUsage(PortfolioMarginUsage { account_value: 100.0, usage: 0.5 }),
                PortfolioStatistic::AllTimePnl(50.0),
                PortfolioStatistic::Volume(5000.0),
            ]
        );
        assert_eq!(data.available_periods, vec![ChartPeriod::Day, ChartPeriod::Year, ChartPeriod::All]);

        let without_summary = perpetual_portfolio_data(PerpetualPortfolio::mock(), ChartPeriod::Week);
        assert!(without_summary.statistics.is_empty());
        assert!(without_summary.charts.iter().all(|chart| chart.values.is_empty()));
    }

    #[test]
    fn test_perpetual_value_chart_drops_the_leading_zero_balance() {
        let date = Utc::now();
        let mut portfolio = PerpetualPortfolio::mock();
        portfolio.day = Some(PerpetualPortfolioTimeframeData {
            account_value_history: vec![
                ChartDateValue { date, value: 0.0 },
                ChartDateValue { date, value: 10.0 },
                ChartDateValue { date, value: 0.0 },
            ],
            pnl_history: vec![],
            volume: 0.0,
        });

        let data = perpetual_portfolio_data(portfolio, ChartPeriod::Day);

        let values = data
            .charts
            .iter()
            .find(|chart| chart.chart_type == PortfolioChartType::Value)
            .map(|chart| chart.values.iter().map(|value| value.value).collect::<Vec<_>>())
            .unwrap();
        assert_eq!(values, vec![10.0, 0.0]);
    }
}
