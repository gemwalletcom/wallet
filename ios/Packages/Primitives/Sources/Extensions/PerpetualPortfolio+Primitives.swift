// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension PerpetualPortfolio {
    var availablePeriods: [ChartPeriod] {
        [(day, ChartPeriod.day), (week, .week), (month, .month), (allTime, .all)].compactMap { $0.0 != nil ? $0.1 : nil }
    }

    func timeframeData(for period: ChartPeriod) -> PerpetualPortfolioTimeframeData? {
        switch period {
        case .hour, .day: day
        case .week: week
        case .month: month
        case .year, .all: allTime
        }
    }
}
