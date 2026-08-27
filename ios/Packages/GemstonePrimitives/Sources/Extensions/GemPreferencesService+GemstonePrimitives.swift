// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Primitives

public extension GemPreferencesServiceProtocol {
    func defaultCurrency(locale: Locale) throws -> Primitives.Currency {
        try Primitives.Currency(defaultCurrency(localeCurrency: locale.currency?.identifier))
    }

}
