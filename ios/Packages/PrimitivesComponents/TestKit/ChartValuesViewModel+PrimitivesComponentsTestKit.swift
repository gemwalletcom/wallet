// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemChartCurrent
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import SwiftUI

public extension ChartValuesViewModel {
    static func mock(
        period: ChartPeriod = .day,
        baseValue: Double = 100,
        current: GemChartCurrent? = GemChartCurrent(date: .now, value: 150, changePercentage: 50),
        values: ChartValues = .mock(),
        type: ChartValueType = .price,
        headerValue: Double? = nil,
    ) -> ChartValuesViewModel {
        ChartValuesViewModel(
            period: period,
            baseValue: baseValue,
            current: current,
            values: values,
            formatter: CurrencyFormatter(type: .currency, currencyCode: "USD"),
            type: type,
            headerValue: headerValue,
        )
    }
}
