// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemHeaderAmount
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension GemHeaderAmount {
    static func mock(
        asset: Asset = .mock(),
        amount: GemFormattedNumber = .mock(value: 1, unit: .symbol(symbol: "BTC"), display: .number(precision: .fraction(min: 0, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest),
        fiat: GemFormattedNumber? = .mock(value: 2, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest),
    ) -> GemHeaderAmount {
        GemHeaderAmount(asset: asset.toGem(), amount: amount, fiat: fiat)
    }
}
