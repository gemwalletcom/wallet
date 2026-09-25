// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemHeaderAmount
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public extension GemHeaderAmount {
    static func mock(
        asset: Asset = .mock(),
        amount: GemFormattedNumber = .mock(unit: .symbol(symbol: "BTC"), display: .number(precision: .fraction(min: 0, max: 2)), notation: .plain),
        fiat: GemFormattedNumber? = .mock(value: 2, notation: .plain),
    ) -> GemHeaderAmount {
        GemHeaderAmount(asset: asset.toGem(), amount: amount, fiat: fiat)
    }
}
