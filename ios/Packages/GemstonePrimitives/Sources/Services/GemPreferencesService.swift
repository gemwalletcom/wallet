// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Primitives

public extension GemPreferencesServiceProtocol {
    var currency: Primitives.Currency {
        getCurrency().toPrimitives()
    }

    func setCurrencyValue(_ currency: Primitives.Currency) throws {
        try setCurrency(currency: currency.toGem())
    }

    var appearanceValue: Primitives.Appearance {
        getAppearance().toPrimitives()
    }

    func setAppearanceValue(_ appearance: Primitives.Appearance) throws {
        try setAppearance(appearance: appearance.toGem())
    }
}
