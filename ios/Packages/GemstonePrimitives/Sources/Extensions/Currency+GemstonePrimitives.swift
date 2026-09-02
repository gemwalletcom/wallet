// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import typealias Gemstone.Currency
import Primitives

public extension Primitives.Currency {
    init(core: Gemstone.Currency) {
        guard let currency = Primitives.Currency(rawValue: core) else {
            fatalError("Core returned a currency this build does not know: \(core)")
        }
        self = currency
    }
}
