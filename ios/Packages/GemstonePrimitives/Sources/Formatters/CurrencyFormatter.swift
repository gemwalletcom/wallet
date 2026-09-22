// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.formattedCurrency
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

    public func string(_ value: Double) -> String {
        Gemstone.formattedCurrency(value: value, code: currencyCode, style: type).text(locale: locale)
    }
}
