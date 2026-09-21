// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct AbbreviatedFormatter {
    private let locale: Locale

    public init(locale: Locale = .current) {
        self.locale = locale
    }

    public func string(from double: Double, rule: FloatingPointRoundingRule = .towardZero) -> String? {
        string(from: Decimal(double), rule: rule)
    }

    public func string(from double: Double, currency: String, rule: FloatingPointRoundingRule = .towardZero) -> String? {
        string(from: Decimal(double), currency: currency, rule: rule)
    }

    public func string(from decimal: Decimal, rule: FloatingPointRoundingRule = .towardZero) -> String? {
        guard #available(iOS 18, *) else {
            return nil
        }

        return decimal.formatted(
            .number
                .notation(.compactName)
                .locale(locale)
                .precision(.fractionLength(0 ... 2))
                .rounded(rule: rule),
        )
    }

    public func string(from decimal: Decimal, currency: String, rule: FloatingPointRoundingRule = .towardZero) -> String? {
        guard #available(iOS 18, *) else {
            return nil
        }

        return decimal.formatted(
            .currency(code: currency)
                .notation(.compactName)
                .locale(locale)
                .precision(.fractionLength(0 ... 2))
                .rounded(rule: rule),
        )
    }
}
