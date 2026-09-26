use primitives::{
    AssetData, ChartPeriod, ChartValuePercentage, Currency, PerpetualPortfolio, PerpetualPortfolioTimeframeData, PortfolioAsset, PortfolioAssets, PortfolioChartData, PortfolioChartType, PortfolioData, PortfolioMarginUsage,
    PortfolioStatistic, PortfolioType,
};

use super::model::GemPortfolioValues;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::list::{GemListRow, GemListRowTitle};
use crate::percentage::GemPercentageStyle;
use crate::precision::GemCurrencyStyle;
use crate::services::balance::model::GemAssetBalance;
use crate::services::chart::GemChartData;
use crate::services::chart::rules::{change_chart_data, converted_values};
use crate::services::localization::GemLocalizedText;

pub fn portfolio_asset(data: &AssetData) -> PortfolioAsset {
    PortfolioAsset {
        asset_id: data.asset.id.clone(),
        value: GemAssetBalance::from(data).total(),
    }
}

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

pub fn fallback_period(period: ChartPeriod, offered: &[ChartPeriod]) -> Option<ChartPeriod> {
    match offered.is_empty() || offered.contains(&period) {
        true => None,
        false => offered.first().copied(),
    }
}

pub fn wallet_periods() -> Vec<ChartPeriod> {
    vec![ChartPeriod::Day, ChartPeriod::Week, ChartPeriod::Month, ChartPeriod::Year, ChartPeriod::All]
}

pub fn portfolio_currency(portfolio_type: PortfolioType, currency: Currency) -> Currency {
    match portfolio_type {
        PortfolioType::Perpetuals => Currency::USD,
        PortfolioType::Wallet => currency,
    }
}

pub fn portfolio_chart_data(data: PortfolioData, portfolio_type: PortfolioType, chart_type: PortfolioChartType, currency: Currency) -> Option<GemChartData> {
    let chart = data.charts.iter().find(|chart| chart.chart_type == chart_type).or(data.charts.first())?;
    let shows_value = portfolio_type == PortfolioType::Wallet || chart_type == PortfolioChartType::Value;
    change_chart_data(chart.values.clone(), shows_value, portfolio_currency(portfolio_type, currency))
}

/// Every statistic finished as a row, so neither app formats a bare f64.
pub fn statistic_rows(statistics: Vec<PortfolioStatistic>, currency: Currency) -> Vec<GemListRow> {
    statistics
        .into_iter()
        .map(|statistic| match statistic {
            PortfolioStatistic::AllTimeHigh { value } => all_time_row(GemListRowTitle::AllTimeHigh, value, currency.clone()),
            PortfolioStatistic::AllTimeLow { value } => all_time_row(GemListRowTitle::AllTimeLow, value, currency.clone()),
            PortfolioStatistic::UnrealizedPnl { value } => amount_row(GemListRowTitle::UnrealizedPnl, GemFormattedNumber::signed_usd(value)),
            PortfolioStatistic::AllTimePnl { value } => amount_row(GemListRowTitle::AllTimePnl, GemFormattedNumber::signed_usd(value)),
            PortfolioStatistic::AccountLeverage { value } => amount_row(GemListRowTitle::AccountLeverage, GemFormattedNumber::leverage(value)),
            PortfolioStatistic::Volume { value } => amount_row(GemListRowTitle::Volume, GemFormattedNumber::usd(value)),
            PortfolioStatistic::MarginUsage { value } => GemListRow::Label {
                title: GemListRowTitle::MarginUsage,
                text: GemLocalizedText::Pnl {
                    amount: GemFormattedNumber::usd(value.used_value),
                    percent: GemFormattedNumber::percentage(value.usage_percent, GemPercentageStyle::Unsigned),
                },
                tone: GemValueTone::Plain,
                info: None,
                progress: false,
            },
        })
        .collect()
}

fn all_time_row(title: GemListRowTitle, value: ChartValuePercentage, currency: Currency) -> GemListRow {
    GemListRow::AllTime {
        title,
        value: GemFormattedNumber::currency(value.value as f64, currency, GemCurrencyStyle::Currency),
        date: value.date,
        change: GemFormattedNumber::percentage(value.percentage as f64, GemPercentageStyle::Signed),
    }
}

fn amount_row(title: GemListRowTitle, amount: GemFormattedNumber) -> GemListRow {
    GemListRow::Amount { title, amount, info: None }
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
            values: timeframe.map(|data| data.account_value_history.iter().skip_while(|value| value.value == 0.0).cloned().collect()).unwrap_or_default(),
        },
    ];

    let mut statistics = Vec::new();
    if let Some(summary) = &portfolio.account_summary {
        statistics.push(PortfolioStatistic::UnrealizedPnl { value: summary.unrealized_pnl });
        statistics.push(PortfolioStatistic::AccountLeverage { value: summary.account_leverage });
        statistics.push(PortfolioStatistic::MarginUsage {
            value: PortfolioMarginUsage::new(summary.account_value, summary.margin_usage),
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
    use chrono::Utc;
    use primitives::{Asset, Balance, ChartDateValue, ChartValue, PerpetualAccountSummary};

    use super::*;

    #[test]
    fn test_the_portfolio_value_is_the_balance_total_core_already_owns() {
        let data = AssetData::mock(
            Asset::mock_eth(),
            Balance {
                staked: 20u32.into(),
                reserved: 7u32.into(),
                ..Balance::coin_balance(100u32.into())
            },
        );

        assert_eq!(portfolio_asset(&data).value, GemAssetBalance::from(&data).total());
        assert_eq!(portfolio_asset(&data).value, 120u32.into(), "a reserved balance is not part of the total");
    }

    #[test]
    fn test_a_perpetuals_portfolio_is_quoted_in_dollars() {
        assert_eq!(portfolio_currency(PortfolioType::Perpetuals, Currency::EUR), Currency::USD, "perpetual collateral is dollars whatever the wallet is set to");
        assert_eq!(portfolio_currency(PortfolioType::Wallet, Currency::EUR), Currency::EUR);
    }

    #[test]
    fn test_every_statistic_leaves_core_as_a_finished_row() {
        let rows = statistic_rows(
            vec![
                PortfolioStatistic::AllTimeHigh {
                    value: ChartValuePercentage {
                        value: 120.0,
                        percentage: -10.0,
                        date: Utc::now(),
                    },
                },
                PortfolioStatistic::UnrealizedPnl { value: -5.0 },
                PortfolioStatistic::AccountLeverage { value: 3.0 },
                PortfolioStatistic::Volume { value: 1_000.0 },
                PortfolioStatistic::MarginUsage {
                    value: PortfolioMarginUsage::new(4.3456, 0.125),
                },
            ],
            Currency::EUR,
        );

        assert!(matches!(&rows[0], GemListRow::AllTime { title: GemListRowTitle::AllTimeHigh, value, .. } if value.value == 120.0));
        assert!(
            matches!(&rows[1], GemListRow::Amount { title: GemListRowTitle::UnrealizedPnl, amount, .. } if amount.tone == GemValueTone::Negative),
            "a loss reads as a loss without either app deciding"
        );
        assert!(matches!(&rows[2], GemListRow::Amount { title: GemListRowTitle::AccountLeverage, .. }));
        assert!(matches!(&rows[3], GemListRow::Amount { title: GemListRowTitle::Volume, .. }));
        assert!(
            matches!(&rows[4], GemListRow::Label { title: GemListRowTitle::MarginUsage, text: GemLocalizedText::Pnl { amount, percent }, .. }
                if amount.value == 0.5432 && percent.value == 12.5),
            "the margin usage carries its amount and percent, not a string either app builds"
        );
    }

    #[test]
    fn test_portfolio_chart_data_picks_the_chart_the_screen_asked_for() {
        let data = PortfolioData {
            charts: vec![
                PortfolioChartData {
                    chart_type: PortfolioChartType::Pnl,
                    values: vec![ChartDateValue::mock(1, 1.0), ChartDateValue::mock(2, 3.0)],
                },
                PortfolioChartData {
                    chart_type: PortfolioChartType::Value,
                    values: vec![ChartDateValue::mock(1, 10.0), ChartDateValue::mock(2, 12.0)],
                },
            ],
            statistics: vec![],
            available_periods: wallet_periods(),
        };

        let pnl = portfolio_chart_data(data.clone(), PortfolioType::Perpetuals, PortfolioChartType::Pnl, Currency::USD).expect("series");
        assert_eq!(pnl.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![1.0, 3.0]);
        assert!(!pnl.shows_secondary_value);

        let value = portfolio_chart_data(data, PortfolioType::Perpetuals, PortfolioChartType::Value, Currency::USD).expect("series");
        assert_eq!(value.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![10.0, 12.0]);
        assert_eq!(value.header.unwrap().secondary_value.map(|value| value.value), Some(12.0));
    }

    #[test]
    fn test_portfolio_chart_data_without_a_chart_has_no_series() {
        let data = PortfolioData {
            charts: vec![],
            statistics: vec![],
            available_periods: wallet_periods(),
        };
        assert_eq!(portfolio_chart_data(data, PortfolioType::Wallet, PortfolioChartType::Value, Currency::USD), None);
    }

    #[test]
    fn test_converted_portfolio_applies_rate_to_values_and_extremes() {
        let portfolio = PortfolioAssets {
            total_value: 10.0,
            values: vec![ChartValue { timestamp: 2, value: 2.0 }, ChartValue { timestamp: 1, value: 1.0 }],
            all_time_high: Some(ChartValuePercentage {
                value: 4.0,
                percentage: 10.0,
                ..ChartValuePercentage::mock()
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

        assert_eq!(data.charts.iter().map(|chart| chart.chart_type).collect::<Vec<_>>(), vec![PortfolioChartType::Pnl, PortfolioChartType::Value]);
        assert_eq!(
            data.statistics,
            vec![
                PortfolioStatistic::UnrealizedPnl { value: 7.0 },
                PortfolioStatistic::AccountLeverage { value: 2.0 },
                PortfolioStatistic::MarginUsage {
                    value: PortfolioMarginUsage::new(100.0, 0.5)
                },
                PortfolioStatistic::AllTimePnl { value: 50.0 },
                PortfolioStatistic::Volume { value: 5000.0 },
            ]
        );
        let margin = PortfolioMarginUsage::new(100.0, 0.5);
        assert_eq!(margin.used_value, 50.0);
        assert_eq!(margin.usage_percent, 50.0);

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
            account_value_history: vec![ChartDateValue { date, value: 0.0 }, ChartDateValue { date, value: 10.0 }, ChartDateValue { date, value: 0.0 }],
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
