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

public extension PriceAlert {
    var type: PriceAlertNotificationType {
        PriceAlertFormatter.shared.notificationType(alert: toGem()).toPrimitives()
    }
}

public extension [PriceAlert] {
    var displayedAlerts: [PriceAlert] {
        PriceAlertFormatter.shared
            .displayedAlertIds(alerts: map { $0.toGem() })
            .compactMap { id in first { $0.id == id } }
    }
}

public extension [PriceAlertData] {
    var displayedAlerts: [PriceAlertData] {
        PriceAlertFormatter.shared
            .displayedAlertIds(alerts: map { $0.priceAlert.toGem() })
            .compactMap { id in first { $0.priceAlert.id == id } }
    }
}
