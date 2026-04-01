// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives

public extension ChartPeriod {
    var title: String {
        switch self {
        case .hour: Localized.Charts.hour
        case .day: Localized.Charts.day
        case .week: Localized.Charts.week
        case .month: Localized.Charts.month
        case .year: Localized.Charts.year
        case .all: Localized.Charts.all
        }
    }
}
