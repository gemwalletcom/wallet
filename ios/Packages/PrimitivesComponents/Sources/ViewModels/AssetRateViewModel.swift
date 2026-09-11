// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import struct Gemstone.GemSwapRate

public struct AssetRateViewModel {
    private let rate: GemSwapRate
    private let formatter: NumericFormatter

    public init(rate: GemSwapRate, formatter: NumericFormatter = NumericFormatter()) {
        self.rate = rate
        self.formatter = formatter
    }

    public func text(isInverse: Bool) -> String {
        let rate = isInverse ? rate.inverse : rate.direct
        return "1 \(rate.baseSymbol) ≈ \(formatter.string(rate.value, symbol: rate.quoteSymbol))"
    }
}
