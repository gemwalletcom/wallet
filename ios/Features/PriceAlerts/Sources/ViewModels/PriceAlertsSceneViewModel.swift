// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class PriceAlertsSceneViewModel: Sendable {
    private let service: any GemPriceAlertServiceProtocol

    public let query: ObservableQuery<PriceAlertsRequest>
    var priceAlerts: [PriceAlertData] {
        query.value
    }

    var isPriceAlertsEnabled: Bool

    public init(
        service: any GemPriceAlertServiceProtocol,
    ) {
        self.service = service
        isPriceAlertsEnabled = service.isEnabled()
        query = ObservableQuery(PriceAlertsRequest(), initialValue: [])
    }

    var title: String {
        Localized.Settings.PriceAlerts.title
    }

    var currencyCode: String {
        service.currency()
    }

    var enableTitle: String {
        Localized.Settings.enableValue("")
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .priceAlerts)
    }

    func sections(for alerts: [PriceAlertData]) -> PriceAlertsSections {
        let (autoAlerts, manualGroups) = alerts.displayedAlerts.reduce(into: ([PriceAlertData](), [Asset: [PriceAlertData]]())) { result, alert in
            switch alert.priceAlert.type {
            case .auto:
                result.0.append(alert)
            case .price, .pricePercentChange:
                result.1[alert.asset, default: []].append(alert)
            }
        }

        return PriceAlertsSections(
            autoAlerts: autoAlerts,
            manualAlerts: manualGroups,
        )
    }
}

// MARK: - Business Logic

extension PriceAlertsSceneViewModel {
    public func load() async {
        do {
            try await service.sync(assetId: nil)
        } catch {
            debugLog("getPriceAlerts error: \(error)")
        }
    }

    func deletePriceAlert(priceAlert: PriceAlert) async {
        do {
            try await service.delete(priceAlerts: [priceAlert])
        } catch {
            debugLog("deletePriceAlert error: \(error)")
        }
    }

    func handleAlertsEnabled(enabled: Bool) async {
        do {
            try await service.setEnabled(enabled: enabled)
        } catch {
            isPriceAlertsEnabled = service.isEnabled()
            debugLog("setPriceAlertsEnabled error: \(error)")
        }
    }
}
