// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import Primitives
@testable import PrimitivesComponents

public extension ChartHeaderViewModel {
    static func mock(
        period: ChartPeriod = .day,
        date: Date? = nil,
        header: GemChartHeader = .mock(),
        valueType: GemChartValueType = .price,
    ) -> ChartHeaderViewModel {
        ChartHeaderViewModel(
            period: period,
            date: date,
            header: header,
            valueType: valueType,
            formatter: CurrencyFormatter(type: .currency, currencyCode: "USD"),
        )
    }
}

public extension GemChartHeader {
    static func mock(
        value: Double = 100,
        secondaryValue: Double? = nil,
        changePercentage: Double? = 5,
    ) -> GemChartHeader {
        GemChartHeader(value: value, secondaryValue: secondaryValue, changePercentage: changePercentage)
    }
}
