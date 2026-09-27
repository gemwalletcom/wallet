// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct CurrencyFormatter: Sendable, Hashable {
    public let locale: Locale
    public let currencyCode: String

    public init(
        locale: Locale = Locale.current,
        currencyCode: String,
    ) {
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
