use primitives::{
    ChartPeriod, ChartValuePercentage, Currency, PerpetualPortfolio, PerpetualPortfolioTimeframeData, PortfolioAssets, PortfolioChartData, PortfolioChartType,
    PortfolioData, PortfolioMarginUsage, PortfolioStatistic, PortfolioType,
};

use super::model::GemPortfolioValues;
use crate::services::chart::GemChartData;
use crate::services::chart::rules::{change_chart_data, converted_values};

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

fn wallet_periods() -> Vec<ChartPeriod> {
    vec![ChartPeriod::Day, ChartPeriod::Week, ChartPeriod::Month, ChartPeriod::Year, ChartPeriod::All]
}

pub fn portfolio_currency(portfolio_type: PortfolioType, currency: Currency) -> Currency {
    match portfolio_type {
        PortfolioType::Perpetuals => Currency::USD,
        PortfolioType::Wallet => currency,
    }
}

pub fn portfolio_chart_data(data: PortfolioData, portfolio_type: PortfolioType, chart_type: PortfolioChartType) -> Option<GemChartData> {
    let chart = data.charts.iter().find(|chart| chart.chart_type == chart_type).or(data.charts.first())?;
    let shows_value = portfolio_type == PortfolioType::Wallet || chart_type == PortfolioChartType::Value;
    change_chart_data(chart.values.clone(), shows_value)
}

pub fn wallet_portfolio_data(values: GemPortfolioValues) -> PortfolioData {
    let statistics = [
        values.all_time_high.map(|value| PortfolioStatistic::AllTimeHigh { value }),
        values.all_time_low.map(|value| PortfolioStatistic::AllTimeLow { value }),
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
        statistics.push(PortfolioStatistic::UnrealizedPnl { value: summary.unrealized_pnl });
        statistics.push(PortfolioStatistic::AccountLeverage { value: summary.account_leverage });
        statistics.push(PortfolioStatistic::MarginUsage {
            value: PortfolioMarginUsage {
                account_value: summary.account_value,
                usage: summary.margin_usage,
            },
        });
    }
    if let Some(all_time) = &portfolio.all_time {
        if let Some(last) = all_time.pnl_history.last() {
            statistics.push(PortfolioStatistic::AllTimePnl { value: last.value });
        }
        statistics.push(PortfolioStatistic::Volume { value: all_time.volume });
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
    use chrono::{DateTime, Utc};
    use primitives::{ChartDateValue, ChartValue, PerpetualAccountSummary};

    use super::*;

    #[test]
    fn test_a_perpetuals_portfolio_is_quoted_in_dollars() {
        assert_eq!(portfolio_currency(PortfolioType::Perpetuals, Currency::EUR), Currency::USD, "perpetual collateral is dollars whatever the wallet is set to");
        assert_eq!(portfolio_currency(PortfolioType::Wallet, Currency::EUR), Currency::EUR);
    }

    #[test]
    fn test_portfolio_chart_data_picks_the_chart_the_screen_asked_for() {
        let point = |seconds: i64, value: f64| ChartDateValue {
            date: DateTime::from_timestamp(seconds, 0).unwrap(),
            value,
        };
        let data = PortfolioData {
            charts: vec![
                PortfolioChartData {
                    chart_type: PortfolioChartType::Pnl,
                    values: vec![point(1, 1.0), point(2, 3.0)],
                },
                PortfolioChartData {
                    chart_type: PortfolioChartType::Value,
                    values: vec![point(1, 10.0), point(2, 12.0)],
                },
            ],
            statistics: vec![],
            available_periods: wallet_periods(),
        };

        let pnl = portfolio_chart_data(data.clone(), PortfolioType::Perpetuals, PortfolioChartType::Pnl).expect("series");
        assert_eq!(pnl.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![1.0, 3.0]);
        assert!(!pnl.shows_secondary_value);

        let value = portfolio_chart_data(data, PortfolioType::Perpetuals, PortfolioChartType::Value).expect("series");
        assert_eq!(value.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![10.0, 12.0]);
        assert_eq!(value.header.unwrap().secondary_value, Some(12.0));
    }

    #[test]
    fn test_portfolio_chart_data_without_a_chart_has_no_series() {
        let data = PortfolioData {
            charts: vec![],
            statistics: vec![],
            available_periods: wallet_periods(),
        };
        assert_eq!(portfolio_chart_data(data, PortfolioType::Wallet, PortfolioChartType::Value), None);
    }

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
                PortfolioStatistic::UnrealizedPnl { value: 7.0 },
                PortfolioStatistic::AccountLeverage { value: 2.0 },
                PortfolioStatistic::MarginUsage {
                    value: PortfolioMarginUsage { account_value: 100.0, usage: 0.5 }
                },
                PortfolioStatistic::AllTimePnl { value: 50.0 },
                PortfolioStatistic::Volume { value: 5000.0 },
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
