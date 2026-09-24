// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemChartBounds
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct ChartValuesViewModel: Sendable {
    public let period: ChartPeriod
    public let lineColor: Color
    let charts: [ChartDateValue]

    private let chartData: GemChartData
    let bounds: GemChartBounds
    private let dateFormatter = ChartDateFormatter()

    public init(
        period: ChartPeriod,
        chartData: GemChartData,
        lineColor: Color = Colors.blue,
    ) {
        self.period = period
        self.chartData = chartData
        self.lineColor = lineColor
        charts = chartData.values.map { $0.toPrimitives() }
        bounds = chartData.bounds()
    }

    var yScale: [Double] {
        [bounds.yMin, bounds.yMax]
    }

    var xScale: [Date] {
        guard let first = charts.first?.date, let last = charts.last?.date else { return [] }
        return [first, last.addingTimeInterval(last.timeIntervalSince(first) * 0.02)]
    }

    var lowerBoundDate: Date {
        charts[Int(bounds.lowerIndex)].date
    }

    var upperBoundDate: Date {
        charts[Int(bounds.upperIndex)].date
    }

    var chartHeader: GemChartHeader? {
        chartData.header
    }

    func header(for element: ChartDateValue) -> GemChartHeader {
        chartData.headerAt(value: element.value)
    }

    func dateText(for element: ChartDateValue) -> String {
        dateFormatter.string(for: element.date, period: period)
    }
}
