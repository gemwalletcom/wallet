// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct NumericFormatter: Sendable, Hashable {
    private let locale: Locale

    public init(locale: Locale = .current) {
        self.locale = locale
    }

    public func string(_ value: Double, symbol: String? = nil) -> String {
        let number = value.formatted(.number.locale(locale).precision(.adaptive(for: abs(value))))
        guard let symbol else { return number }
        return "\(number) \(symbol)"
    }

    public func double(from amount: String) -> Double? {
        Decimal(string: amount, locale: locale)?.doubleValue
    }
}
