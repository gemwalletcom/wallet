// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.PriceAlertFormatter
import Primitives

extension PriceAlert: @retroactive Identifiable {
    public var id: String {
        PriceAlertFormatter.shared.alertId(alert: toGem())
    }
}

extension PriceAlertData: @retroactive Identifiable {
    public var id: String {
        asset.id.identifier + priceAlert.id
    }
}
