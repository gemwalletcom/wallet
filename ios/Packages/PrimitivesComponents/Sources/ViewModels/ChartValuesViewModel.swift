// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import func Gemstone.chartHeader
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct ChartValuesViewModel: Sendable {
    public let period: ChartPeriod
    public let lineColor: Color
    public let values: ChartValues

    private let chartData: GemChartData
    private let formatter: CurrencyFormatter

    public init?(
        period: ChartPeriod,
        chartData: GemChartData,
        lineColor: Color = Colors.blue,
        formatter: CurrencyFormatter,
    ) {
        guard let values = try? ChartValues.from(charts: chartData.values.map { $0.map() }) else {
            return nil
        }
        self.period = period
        self.chartData = chartData
        self.lineColor = lineColor
        self.formatter = formatter
        self.values = values
    }

    var charts: [ChartDateValue] {
        values.charts
    }

    var lowerBoundValueText: String {
        formatter.string(values.lowerBoundValue)
    }

    var upperBoundValueText: String {
        formatter.string(values.upperBoundValue)
    }

    var chartHeaderViewModel: ChartHeaderViewModel? {
        chartData.header.map { headerViewModel($0, date: nil) }
    }

    func headerViewModel(for element: ChartDateValue) -> ChartHeaderViewModel {
        let header = chartHeader(
            valueType: chartData.valueType,
            base: chartData.base,
            value: element.value,
            showsSecondaryValue: chartData.showsSecondaryValue,
        )
        return headerViewModel(header, date: element.date)
    }

    private func headerViewModel(_ header: GemChartHeader, date: Date?) -> ChartHeaderViewModel {
        ChartHeaderViewModel(period: period, date: date, header: header, valueType: chartData.valueType, formatter: formatter)
    }
}
