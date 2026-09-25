// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents

public extension ChartValuesViewModel {
    static func mock(
        period: ChartPeriod = .day,
        chartData: GemChartData = .mock(),
    ) -> ChartValuesViewModel {
        ChartValuesViewModel(period: period, chartData: chartData)
    }
}
