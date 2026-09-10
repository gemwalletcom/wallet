// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import enum Gemstone.GemAmountSign
import Primitives
@testable import PrimitivesComponents

public extension AmountDisplayStyle {
    static func mock(
        sign: GemAmountSign = .none,
        formatter: ValueFormatter = .full,
        currencyCode: String = "USD",
    ) -> AmountDisplayStyle {
        AmountDisplayStyle(
            sign: sign,
            formatter: formatter,
            currencyCode: currencyCode,
        )
    }
}
