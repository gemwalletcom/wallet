// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.ChartDateValue
import enum Gemstone.Currency
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import SwiftUI

public extension ChartValuesViewModel {
    static func mock(
        period: ChartPeriod = .day,
        chartData: GemChartData = .mock(),
    ) -> ChartValuesViewModel {
        ChartValuesViewModel(period: period, chartData: chartData)
    }
}

public extension GemChartData {
    static func mock(
        values: [Double] = [100, 150, 80, 120],
        header: GemChartHeader? = .mock(),
        valueType: GemChartValueType = .price,
        base: Double? = nil,
        showsSecondaryValue: Bool = false,
        currency: Gemstone.Currency = Primitives.Currency.usd.toGem(),
    ) -> GemChartData {
        GemChartData(
            valueType: valueType,
            base: base ?? values.first ?? 0,
            showsSecondaryValue: showsSecondaryValue,
            currency: currency,
            values: values.enumerated().map {
                Gemstone.ChartDateValue(date: Date(timeIntervalSince1970: Double($0.offset) * 3600), value: $0.element)
            },
            header: header,
        )
    }
}
