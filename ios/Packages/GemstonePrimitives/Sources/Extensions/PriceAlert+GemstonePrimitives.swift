// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.PriceAlertFormatter
import Primitives

private let priceAlertFormatter = PriceAlertFormatter()

extension PriceAlert: @retroactive Identifiable {
    public var id: String {
        priceAlertFormatter.alertId(alert: json())
    }
}

extension PriceAlertData: @retroactive Identifiable {
    public var id: String {
        asset.id.identifier + priceAlert.id
    }
}

public extension PriceAlert {
    var type: PriceAlertNotificationType {
        priceAlertFormatter.notificationType(alert: json()).map()
    }
}

public extension [PriceAlert] {
    var displayedAlerts: [PriceAlert] {
        priceAlertFormatter
            .displayedAlertIds(alerts: map { $0.json() })
            .compactMap { id in first { $0.id == id } }
    }
}

public extension [PriceAlertData] {
    var displayedAlerts: [PriceAlertData] {
        priceAlertFormatter
            .displayedAlertIds(alerts: map { $0.priceAlert.json() })
            .compactMap { id in first { $0.priceAlert.id == id } }
    }
}

public extension AssetData {
    var isPriceAlertsEnabled: Bool {
        priceAlerts.contains { $0.type == .auto }
    }
}
