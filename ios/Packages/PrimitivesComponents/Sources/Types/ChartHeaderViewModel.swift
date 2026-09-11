// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import Primitives
import Style
import SwiftUI

public struct ChartHeaderViewModel {
    public let period: ChartPeriod
    public let date: Date?
    public let header: GemChartHeader
    public let valueType: GemChartValueType

    private let formatter: CurrencyFormatter
    private let dateFormatter: ChartDateFormatter

    public init(
        period: ChartPeriod,
        date: Date?,
        header: GemChartHeader,
        valueType: GemChartValueType = .price,
        formatter: CurrencyFormatter,
        dateFormatter: ChartDateFormatter = ChartDateFormatter(),
    ) {
        self.period = period
        self.date = date
        self.header = header
        self.valueType = valueType
        self.formatter = formatter
        self.dateFormatter = dateFormatter
    }

    private var valueChange: PriceChangeViewModel? {
        valueType == .priceChange ? PriceChangeViewModel(value: header.value, currencyFormatter: formatter) : nil
    }

    public var dateText: String? {
        date.map { dateFormatter.string(for: $0, period: period) }
    }

    public var headerValueText: String? {
        header.secondaryValue.map { formatter.string($0) }
    }

    public var priceText: String {
        valueChange?.text ?? formatter.string(header.value)
    }

    public var priceColor: Color {
        valueChange?.color ?? Colors.black
    }

    public var priceChangeText: String? {
        header.changePercentage.map {
            switch valueType {
            case .priceChange: "(\(PercentFormatter.unsigned.string($0)))"
            case .price: PercentFormatter.signed.string($0)
            }
        }
    }

    public var priceChangeTextColor: Color {
        PriceChangeColor.color(for: header.changePercentage ?? 0)
    }

    public var priceFont: Font {
        header.secondaryValue != nil ? .app.headline : .title2
    }

    public var priceChangeFont: Font {
        header.secondaryValue != nil ? .app.headline : .callout
    }
}
