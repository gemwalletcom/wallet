// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemChartBounds
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import struct Gemstone.GemChartViewport
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct ChartValuesViewModel: Sendable {
    public let period: ChartPeriod
    public let lineColor: Color
    let charts: [ChartDateValue]
    let renderValues: [ChartDateValue]

    private let chartData: GemChartData
    private let viewport: GemChartViewport
    let bounds: GemChartBounds
    private let dateFormatter = ChartDateFormatter()

    public init(
        period: ChartPeriod,
        chartData: GemChartData,
        viewport: GemChartViewport,
        lineColor: Color = Colors.blue,
    ) {
        self.period = period
        self.chartData = chartData
        self.viewport = viewport
        self.lineColor = lineColor
        charts = viewport.values.map { $0.toPrimitives() }
        renderValues = viewport.renderValues.map { $0.toPrimitives() }
        bounds = viewport.bounds
    }

    var yScale: [Double] {
        [bounds.yMin, bounds.yMax]
    }

    var xScale: [Date] {
        [viewport.start, viewport.end.addingTimeInterval(viewport.end.timeIntervalSince(viewport.start) * 0.02)]
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

    func value(for date: Date) -> ChartDateValue? {
        renderValues
            .filter { $0.date >= viewport.start }
            .min { abs($0.date.distance(to: date)) < abs($1.date.distance(to: date)) }
    }
}
