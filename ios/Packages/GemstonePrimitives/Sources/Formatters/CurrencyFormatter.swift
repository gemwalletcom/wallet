// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemCurrencyStyle
import Primitives

public struct CurrencyFormatter: Sendable, Hashable {
    public let locale: Locale
    public let type: GemCurrencyStyle
    public let currencyCode: String

    public init(
        type: GemCurrencyStyle = .currency,
        locale: Locale = Locale.current,
        currencyCode: String,
    ) {
        self.type = type
        self.locale = locale
        self.currencyCode = currencyCode
    }

    public var symbol: String {
        let formatter = NumberFormatter()
        formatter.locale = locale
        formatter.numberStyle = .currency
        formatter.currencyCode = currencyCode
        return formatter.currencySymbol
    }
}
