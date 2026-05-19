// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum CurrencyFormatterType: Sendable, Hashable {
    case currency
    case fiat
    case abbreviated
}

public struct CurrencyFormatter: Sendable, Hashable {
    private let locale: Locale
    private let type: CurrencyFormatterType
    public let currencyCode: String

    public init(
        type: CurrencyFormatterType = .currency,
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
        switch type {
        case .currency, .fiat: currencyString(value)
        case .abbreviated: abbreviatedFormatter.string(from: value, currency: currencyCode) ?? currencyString(value)
        }
    }
}

// MARK: - Private

private extension CurrencyFormatter {
    var abbreviatedFormatter: AbbreviatedFormatter {
        AbbreviatedFormatter(locale: locale)
    }

    func currencyString(_ value: Double) -> String {
        value.formatted(.currency(code: currencyCode).locale(locale).precision(precision(for: abs(value))))
    }

    func precision(for magnitude: Double) -> NumberFormatStyleConfiguration.Precision {
        switch type {
        case .fiat: .twoPlaces
        case .currency, .abbreviated: .adaptive(for: magnitude)
        }
    }
}
