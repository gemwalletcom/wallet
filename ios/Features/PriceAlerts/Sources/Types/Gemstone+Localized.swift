// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPriceAlertPrompt
import Localization

extension GemPriceAlertPrompt {
    var title: String {
        switch self {
        case .targetPrice: Localized.PriceAlerts.SetAlert.setTargetPrice
        case .priceOver: Localized.PriceAlerts.SetAlert.priceOver
        case .priceUnder: Localized.PriceAlerts.SetAlert.priceUnder
        case .increasesBy: Localized.PriceAlerts.SetAlert.priceIncreasesBy
        case .decreasesBy: Localized.PriceAlerts.SetAlert.priceDecreasesBy
        }
    }
}
