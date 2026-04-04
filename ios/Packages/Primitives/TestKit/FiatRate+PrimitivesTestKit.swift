// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension FiatRate {
    static func mock(
        symbol: String = Currency.usd.rawValue,
        rate: Double = 1.0,
    ) -> Self {
        FiatRate(symbol: symbol, rate: rate)
    }
}
