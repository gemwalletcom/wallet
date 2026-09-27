// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemChartData
import Primitives

@MainActor
public protocol ChartListViewable: AnyObject, Observable {
    var chartState: StateViewType<GemChartData> { get }
    var selectedPeriod: ChartPeriod { get set }
    var periods: [ChartPeriod] { get }
    func load() async
}

public extension ChartListViewable {
    var periods: [ChartPeriod] {
        [.hour, .day, .week, .month, .year, .all]
    }
}
