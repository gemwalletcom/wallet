// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
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

    public func string(_ value: Double) -> String {
        guard type.abbreviates(magnitude: value) else {
            return currencyString(value)
        }
        return abbreviatedFormatter.string(from: value, currency: currencyCode) ?? currencyString(value)
    }
}

// MARK: - Private

private extension CurrencyFormatter {
    var abbreviatedFormatter: AbbreviatedFormatter {
        AbbreviatedFormatter(locale: locale)
    }

    func currencyString(_ value: Double) -> String {
        value.formatted(.currency(code: currencyCode).locale(locale).precision(type.precision(magnitude: abs(value)).formatStyle))
    }
}
