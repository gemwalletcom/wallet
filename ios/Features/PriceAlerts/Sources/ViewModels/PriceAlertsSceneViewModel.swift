// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemPriceAlertServiceProtocol
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@Observable
@MainActor
public final class PriceAlertsSceneViewModel: Sendable {
    private let priceAlertService: any GemPriceAlertServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol

    public let query: ObservableQuery<PriceAlertsRequest>
    var priceAlerts: [PriceAlertData] {
        query.value
    }

    var isPriceAlertsEnabled: Bool

    public init(
        priceAlertService: any GemPriceAlertServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
    ) {
        self.priceAlertService = priceAlertService
        self.preferencesService = preferencesService
        isPriceAlertsEnabled = priceAlertService.isEnabled()
        query = ObservableQuery(PriceAlertsRequest(), initialValue: [])
    }

    var title: String {
        Localized.Settings.PriceAlerts.title
    }

    var currencyCode: String {
        preferencesService.currencyCode
    }

    var enableTitle: String {
        Localized.Settings.enableValue("")
    }

    var emptyContentModel: EmptyContentTypeViewModel {
        EmptyContentTypeViewModel(type: .priceAlerts)
    }

    func sections(for alerts: [PriceAlertData]) -> PriceAlertsSections {
        let (autoAlerts, manualGroups) = alerts.reduce(into: ([PriceAlertData](), [Asset: [PriceAlertData]]())) { result, alert in
            switch alert.priceAlert.type {
            case .auto:
                result.0.append(alert)
            case .price, .pricePercentChange:
                guard alert.priceAlert.lastNotifiedAt == nil else { return }
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
    public func fetch() async {
        do {
            try await priceAlertService.sync(assetId: nil)
        } catch {
            debugLog("getPriceAlerts error: \(error)")
        }
    }

    func deletePriceAlert(priceAlert: PriceAlert) async {
        do {
            try await priceAlertService.delete(priceAlerts: [priceAlert])
        } catch {
            debugLog("deletePriceAlert error: \(error)")
        }
    }

    func handleAlertsEnabled(enabled: Bool) async {
        do {
            try await priceAlertService.setEnabled(enabled: enabled)
        } catch {
            isPriceAlertsEnabled = priceAlertService.isEnabled()
            debugLog("setPriceAlertsEnabled error: \(error)")
        }
    }
}
