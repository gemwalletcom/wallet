// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import struct Gemstone.GemChartViewport

public extension GemChartViewport {
    static func mock(chartData: GemChartData) -> GemChartViewport {
        GemChartViewport(
            start: chartData.values.first?.date ?? Date(timeIntervalSince1970: 0),
            end: chartData.values.last?.date ?? Date(timeIntervalSince1970: 0),
            values: chartData.values,
            renderValues: chartData.values,
            bounds: chartData.bounds(),
        )
    }
}
