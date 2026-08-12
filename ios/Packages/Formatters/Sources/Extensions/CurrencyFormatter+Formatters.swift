// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension CurrencyFormatter {
    static let usd = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)
}
