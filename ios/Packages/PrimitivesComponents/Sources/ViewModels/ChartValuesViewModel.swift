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
    private let bounds: GemChartBounds
    private let formatter: CurrencyFormatter

    public init(
        period: ChartPeriod,
        chartData: GemChartData,
        lineColor: Color = Colors.blue,
    ) {
        self.period = period
        self.chartData = chartData
        self.lineColor = lineColor
        formatter = CurrencyFormatter(currencyCode: chartData.currency.toPrimitives().rawValue)
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

    var lowerBoundValueText: String {
        formatter.string(charts[Int(bounds.lowerIndex)].value)
    }

    var upperBoundValueText: String {
        formatter.string(charts[Int(bounds.upperIndex)].value)
    }

    var chartHeaderViewModel: ChartHeaderViewModel? {
        chartData.header.map { headerViewModel($0, date: nil) }
    }

    func headerViewModel(for element: ChartDateValue) -> ChartHeaderViewModel {
        headerViewModel(chartData.headerAt(value: element.value), date: element.date)
    }

    private func headerViewModel(_ header: GemChartHeader, date: Date?) -> ChartHeaderViewModel {
        ChartHeaderViewModel(period: period, date: date, header: header, valueType: chartData.valueType)
    }
}
