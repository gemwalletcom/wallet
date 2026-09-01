// Copyright (c). Gem Wallet. All rights reserved.

public extension PerpetualPortfolio {
    var availablePeriods: [ChartPeriod] {
        [(day, ChartPeriod.day), (week, .week), (month, .month), (allTime, .year), (allTime, .all)].compactMap { $0.0 != nil ? $0.1 : nil }
    }
}
