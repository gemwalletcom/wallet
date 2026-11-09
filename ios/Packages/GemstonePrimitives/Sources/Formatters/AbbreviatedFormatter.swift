// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

struct AbbreviatedFormatter {
    private let locale: Locale

    init(locale: Locale = .current) {
        self.locale = locale
    }

    func string(from double: Double, rule: FloatingPointRoundingRule = .towardZero) -> String? {
        string(from: Decimal(double), rule: rule)
    }

    func string(from double: Double, currency: String, rule: FloatingPointRoundingRule = .towardZero) -> String? {
        string(from: Decimal(double), currency: currency, rule: rule)
    }

    func string(from decimal: Decimal, rule: FloatingPointRoundingRule = .towardZero) -> String? {
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

    func string(from decimal: Decimal, currency: String, rule: FloatingPointRoundingRule = .towardZero) -> String? {
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
