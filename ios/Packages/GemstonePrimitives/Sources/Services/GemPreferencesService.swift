// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Primitives

public extension GemPreferencesServiceProtocol {
    var currency: Primitives.Currency {
        Primitives.Currency(core: getCurrency())
    }

    func setCurrencyValue(_ currency: Primitives.Currency) throws {
        try setCurrency(currency: currency.rawValue)
    }

    var chartPeriodValue: ChartPeriod {
        getChartPeriod().map()
    }

    func setChartPeriodValue(_ period: ChartPeriod) {
        try? setChartPeriod(period: period.map())
    }

    var appearanceValue: Primitives.Appearance {
        getAppearance().map()
    }

    func setAppearanceValue(_ appearance: Primitives.Appearance) throws {
        try setAppearance(appearance: appearance.map())
    }

    func showPerpetuals(for wallet: Wallet) -> Bool {
        showPerpetuals(wallet: wallet.map())
    }
}
