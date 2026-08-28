// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPriceAlertStore
import typealias Gemstone.PriceAlert
import func Gemstone.priceAlertId
import GemstonePrimitives
import Primitives
import Store

public final class GemstonePriceAlertStore: GemPriceAlertStore, @unchecked Sendable {
    private let store: PriceAlertStore

    public init(store: PriceAlertStore) {
        self.store = store
    }

    public func getPriceAlerts(assetId: String?) async throws -> [Gemstone.PriceAlert] {
        let alerts = try assetId.map { try store.getPriceAlerts(for: $0) } ?? store.getPriceAlerts()
        return try alerts.map { try $0.json() }
    }

    public func updatePriceAlerts(alerts: [Gemstone.PriceAlert], deleteIds: [String]) async throws {
        try store.diffPriceAlerts(
            deleteIds: deleteIds,
            alerts: alerts.map { try (id: priceAlertId(alert: $0), alert: Primitives.PriceAlert($0)) },
        )
    }
}
