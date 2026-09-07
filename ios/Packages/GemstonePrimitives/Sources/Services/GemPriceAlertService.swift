// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPriceAlertServiceProtocol
import Primitives

public extension GemPriceAlertServiceProtocol {
    func enable(priceAlert: PriceAlert) async throws {
        try await enablePriceAlert(alert: priceAlert.map())
    }

    func delete(priceAlerts: [PriceAlert]) async throws {
        try await deletePriceAlerts(alerts: priceAlerts.map { $0.map() })
    }
}
