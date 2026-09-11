// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.ChartDateValue
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import SwiftUI

public extension ChartValuesViewModel {
    static func mock(
        period: ChartPeriod = .day,
        chartData: GemChartData = .mock(),
    ) -> ChartValuesViewModel {
        ChartValuesViewModel(
            period: period,
            chartData: chartData,
            formatter: CurrencyFormatter(type: .currency, currencyCode: "USD"),
        )!
    }
}

public extension GemChartData {
    static func mock(
        values: [Double] = [100, 150, 80, 120],
        header: GemChartHeader? = .mock(),
        valueType: GemChartValueType = .price,
    ) -> GemChartData {
        GemChartData(
            valueType: valueType,
            base: values.first ?? 0,
            showsSecondaryValue: false,
            values: values.enumerated().map {
                Gemstone.ChartDateValue(date: Date(timeIntervalSince1970: Double($0.offset) * 3600), value: $0.element)
            },
            header: header,
        )
    }
}
