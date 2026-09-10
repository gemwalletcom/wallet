// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemChartCurrent
import class Gemstone.PriceChangeCalculator
import Formatters
import Foundation
import Primitives
import Style
import SwiftUI

internal import Charts

public struct ChartValuesViewModel: Sendable {
    private static let priceChangeCalculator = PriceChangeCalculator()
    public let period: ChartPeriod
    public let baseValue: Double
    public let current: GemChartCurrent?
    public let values: ChartValues
    public let lineColor: Color
    public let formatter: CurrencyFormatter
    public let type: ChartValueType
    public let headerValue: Double?

    public init(
        period: ChartPeriod,
        baseValue: Double,
        current: GemChartCurrent?,
        values: ChartValues,
        lineColor: Color = Colors.blue,
        formatter: CurrencyFormatter,
        type: ChartValueType = .price,
        headerValue: Double? = nil,
    ) {
        self.period = period
        self.baseValue = baseValue
        self.current = current
        self.values = values
        self.lineColor = lineColor
        self.formatter = formatter
        self.type = type
        self.headerValue = headerValue
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
        if let current {
            return ChartHeaderViewModel(period: period, date: nil, price: current.value, priceChangePercentage: current.changePercentage, headerValue: headerValue, formatter: formatter, type: type)
        }
        guard let last = charts.last else { return nil }
        return headerViewModel(for: last, date: nil)
    }

    public static func priceChange(
        charts: [ChartDateValue],
        period: ChartPeriod,
        formatter: CurrencyFormatter,
        showHeaderValue: Bool = false,
    ) -> ChartValuesViewModel? {
        guard let values = try? ChartValues.from(charts: charts), values.hasVariation else {
            return nil
        }
        let current = GemChartCurrent(
            date: .now,
            value: values.lastValue - values.firstValue,
            changePercentage: priceChangeCalculator.percentage(from: values.firstValue, to: values.lastValue),
        )
        return ChartValuesViewModel(
            period: period,
            baseValue: values.firstValue,
            current: current,
            values: values,
            formatter: formatter,
            type: .priceChange,
            headerValue: showHeaderValue ? values.lastValue : nil,
        )
    }

    func headerViewModel(for element: ChartDateValue) -> ChartHeaderViewModel {
        headerViewModel(for: element, date: element.date)
    }

    private func headerViewModel(for element: ChartDateValue, date: Date?) -> ChartHeaderViewModel {
        let priceChangePercentage = Self.priceChangeCalculator.percentage(from: baseValue, to: element.value)
        let displayPrice = type == .priceChange ? element.value - baseValue : element.value
        let elementHeaderValue = headerValue != nil ? element.value : nil
        return ChartHeaderViewModel(period: period, date: date, price: displayPrice, priceChangePercentage: priceChangePercentage, headerValue: elementHeaderValue, formatter: formatter, type: type)
    }
}
