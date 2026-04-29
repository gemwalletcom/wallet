// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives

@MainActor
public protocol ChartListViewable: AnyObject, Observable {
    var chartState: StateViewType<ChartValuesViewModel> { get }
    var selectedPeriod: ChartPeriod { get set }
    var periods: [ChartPeriod] { get }
    func fetch() async
}

public extension ChartListViewable {
    var periods: [ChartPeriod] {
        [.hour, .day, .week, .month, .year, .all]
    }
}
