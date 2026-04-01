// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
@testable import Perpetuals
import Primitives

public extension CandleTooltipViewModel {
    static func mock(
        candle: ChartCandleStick = .mock(),
        formatter: CurrencyFormatter = CurrencyFormatter(type: .currency, locale: Locale(identifier: "en_US"), currencyCode: "USD"),
    ) -> CandleTooltipViewModel {
        CandleTooltipViewModel(candle: candle, formatter: formatter)
    }
}
