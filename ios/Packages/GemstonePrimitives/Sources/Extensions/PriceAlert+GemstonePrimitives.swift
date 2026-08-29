// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.PriceAlertFormatter
import Primitives

private let priceAlertFormatter = PriceAlertFormatter()

extension PriceAlert: @retroactive Identifiable {
    public var id: String {
        do {
            return try priceAlertFormatter.alertId(alert: json())
        } catch {
            preconditionFailure("Unencodable price alert: \(error)")
        }
    }
}

extension PriceAlertData: @retroactive Identifiable {
    public var id: String {
        asset.id.identifier + priceAlert.id
    }
}

public extension PriceAlert {
    var type: PriceAlertNotificationType {
        guard let alert = try? json(),
              let type = try? PriceAlertNotificationType(priceAlertFormatter.notificationType(alert: alert))
        else {
            return .auto
        }
        return type
    }

    var shouldDisplay: Bool {
        guard let alert = try? json() else { return true }
        return priceAlertFormatter.shouldDisplay(alert: alert)
    }
}

public extension AssetData {
    var isPriceAlertsEnabled: Bool {
        priceAlerts.contains { $0.type == .auto }
    }
}
