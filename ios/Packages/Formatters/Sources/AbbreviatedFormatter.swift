// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public let defaultAbbreviationThreshold: Decimal = 100000.0

public struct AbbreviatedFormatter {
    private let locale: Locale
    private let threshold: Decimal

    public init(
        locale: Locale = .current,
        threshold: Decimal = defaultAbbreviationThreshold,
    ) {
        self.locale = locale
        self.threshold = threshold
    }

    public func string(from double: Double) -> String? {
        string(from: Decimal(double))
    }

    public func string(from double: Double, currency: String) -> String? {
        string(from: Decimal(double), currency: currency)
    }

    func string(from decimal: Decimal) -> String? {
        guard abs(decimal) >= threshold, #available(iOS 18, *) else {
            return nil
        }

        return decimal.formatted(
            .number
                .notation(.compactName)
                .locale(locale)
                .precision(.significantDigits(1 ... 2)),
        )
    }

    func string(from decimal: Decimal, currency: String) -> String? {
        guard abs(decimal) >= threshold, #available(iOS 18, *) else {
            return nil
        }

        return decimal.formatted(
            .currency(code: currency)
                .notation(.compactName)
                .locale(locale)
                .precision(.significantDigits(1 ... 2)),
        )
    }
}
