// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.adaptivePrecision

public struct NumericFormatter: Sendable, Hashable {
    private let locale: Locale

    public init(locale: Locale = .current) {
        self.locale = locale
    }

    public func string(_ value: Double, symbol: String? = nil) -> String {
        let number = value.formatted(.number.locale(locale).precision(adaptivePrecision(magnitude: abs(value)).formatStyle))
        guard let symbol else { return number }
        return "\(number) \(symbol)"
    }
}
