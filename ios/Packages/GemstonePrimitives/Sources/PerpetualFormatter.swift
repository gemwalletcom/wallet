// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public struct PerpetualFormatter {
    private let perpetual: Gemstone.GemPerpetual

    public init(provider: Primitives.PerpetualProvider) {
        perpetual = Gemstone.GemPerpetual(provider: provider.map())
    }

    public func formatPrice(_ price: Double, decimals: Int32) -> String {
        perpetual.formatPrice(price: price, decimals: decimals)
    }

    public var recipient: GemRecipient {
        perpetual.recipient()
    }

    public func formatInputPrice(_ price: Double, decimals: Int32, locale: Locale = .current) -> String {
        let formatted = perpetual.formatPrice(price: price, decimals: decimals)
        let decimalSeparator = locale.decimalSeparator ?? "."
        guard decimalSeparator != "." else {
            return formatted
        }
        return formatted.replacingOccurrences(of: ".", with: decimalSeparator)
    }
}
