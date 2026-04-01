// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import SwiftUI

public extension ChartValuesViewModel {
    static func mock(
        period: ChartPeriod = .day,
        price: Price? = .mock(),
        values: ChartValues = .mock(),
        type: ChartValueType = .price,
        headerValue: Double? = nil,
    ) -> ChartValuesViewModel {
        ChartValuesViewModel(
            period: period,
            price: price,
            values: values,
            formatter: CurrencyFormatter(type: .currency, currencyCode: "USD"),
            type: type,
            headerValue: headerValue,
        )
    }
}
