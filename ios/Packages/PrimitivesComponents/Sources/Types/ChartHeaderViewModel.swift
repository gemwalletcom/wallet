// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import GemstonePrimitives
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

    private let dateFormatter: ChartDateFormatter

    public init(
        period: ChartPeriod,
        date: Date?,
        header: GemChartHeader,
        valueType: GemChartValueType = .price,
        dateFormatter: ChartDateFormatter = ChartDateFormatter(),
    ) {
        self.period = period
        self.date = date
        self.header = header
        self.valueType = valueType
        self.dateFormatter = dateFormatter
    }

    public var dateText: String? {
        date.map { dateFormatter.string(for: $0, period: period) }
    }

    public var headerValueText: String? {
        header.secondaryValue?.text()
    }

    public var priceText: String {
        header.value.text()
    }

    public var priceColor: Color {
        header.value.tone.color
    }

    public var priceChangeText: String? {
        header.change?.text()
    }

    public var priceChangeTextColor: Color {
        header.change?.tone.color ?? Colors.gray
    }

    public var priceFont: Font {
        header.secondaryValue != nil ? .app.headline : .title2
    }

    public var priceChangeFont: Font {
        header.secondaryValue != nil ? .app.headline : .callout
    }
}
