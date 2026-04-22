// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension ChartValuePercentage {
    static func mock(
        date: Date = Date(),
        value: Float = 1500,
        percentage: Float = 5.0,
    ) -> ChartValuePercentage {
        ChartValuePercentage(date: date, value: value, percentage: percentage)
    }
}

public extension PortfolioMarginUsage {
    static func mock(
        accountValue: Double = 10000,
        usage: Double = 0.15,
    ) -> PortfolioMarginUsage {
        PortfolioMarginUsage(accountValue: accountValue, usage: usage)
    }
}

public extension PortfolioChartData {
    static func mock(
        chartType: PortfolioChartType = .value,
        values: [ChartDateValue] = ChartDateValue.mockHistory(),
    ) -> PortfolioChartData {
        PortfolioChartData(chartType: chartType, values: values)
    }
}

public extension PortfolioData {
    static func mockWallet(
        charts: [PortfolioChartData] = [.mock(chartType: .value)],
        statistics: [PortfolioStatistic] = [
            .allTimeHigh(.mock()),
            .allTimeLow(.mock(value: 800, percentage: -10.0)),
        ],
        availablePeriods: [ChartPeriod] = [.day, .week, .month, .year, .all],
    ) -> PortfolioData {
        PortfolioData(charts: charts, statistics: statistics, availablePeriods: availablePeriods)
    }

    static func mockPerpetual(
        charts: [PortfolioChartData] = [
            .mock(chartType: .pnl, values: ChartDateValue.mockHistory(values: [0, 5, 2, 8, 10])),
            .mock(chartType: .value, values: ChartDateValue.mockHistory(values: [100, 105, 102, 108, 110])),
        ],
        statistics: [PortfolioStatistic] = [
            .unrealizedPnl(500),
            .accountLeverage(2.5),
            .marginUsage(.mock()),
            .allTimePnl(1200),
            .volume(50000),
        ],
        availablePeriods: [ChartPeriod] = [.day, .week, .month, .year, .all],
    ) -> PortfolioData {
        PortfolioData(charts: charts, statistics: statistics, availablePeriods: availablePeriods)
    }
}
