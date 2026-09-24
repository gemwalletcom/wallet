// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public struct PerpetualFormatter {
    private let perpetual: Gemstone.GemPerpetual

    public init(provider: Primitives.PerpetualProvider) {
        perpetual = Gemstone.GemPerpetual(provider: provider.toGem())
    }

    public func formatInputPrice(_ price: Double, decimals: Int32, locale: Locale = .current) -> String {
        perpetual.formatInputPrice(price: price, decimals: decimals, decimalSeparator: NumberInput.format(locale).decimalSeparator)
    }
}
