// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPriceAlertPrompt
import enum Gemstone.GemPriceAlertSectionKind
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

extension GemPriceAlertSectionKind {
    var title: String {
        switch self {
        case .auto: ""
        case let .asset(_, name): name
        }
    }

    var footer: String? {
        switch self {
        case .auto: Localized.PriceAlerts.autoFooter
        case .asset: nil
        }
    }
}
