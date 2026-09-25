// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemHeaderAmount
import GemstonePrimitivesTestKit
import PrimitivesComponents

public extension NumericViewModel {
    static func mock(
        header: GemHeaderAmount = .mock(
            amount: .mock(value: 1, unit: .symbol(symbol: "BTC"), display: .number(precision: .fraction(min: 0, max: 2)), notation: .plain),
            fiat: .mock(value: 2, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain),
        ),
    ) -> NumericViewModel {
        NumericViewModel(header: header)
    }
}
