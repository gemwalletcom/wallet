// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPreferencesServiceProtocol
import Primitives

public extension GemPreferencesServiceProtocol {
    var currency: Primitives.Currency {
        Primitives.Currency(core: getCurrency())
    }

    var currencyCode: String {
        getCurrency()
    }

    func setCurrencyValue(_ currency: Primitives.Currency) throws {
        try setCurrency(currency: currency.rawValue)
    }

    var chartPeriodValue: ChartPeriod {
        (try? ChartPeriod(getChartPeriod())) ?? .day
    }

    func setChartPeriodValue(_ period: ChartPeriod) {
        try? setChartPeriod(period: period.json())
    }

    var appearanceValue: Primitives.Appearance {
        (try? Primitives.Appearance(getAppearance())) ?? .system
    }

    func setAppearanceValue(_ appearance: Primitives.Appearance) throws {
        try setAppearance(appearance: appearance.json())
    }

    func showPerpetuals(for wallet: Wallet) -> Bool {
        showPerpetuals(wallet: wallet.json())
    }
}
