// Copyright (c). Gem Wallet. All rights reserved.

import Components
import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class AssetPriceAlertsViewModel: Sendable {
    let priceAlertService: any GemPriceAlertServiceProtocol
    let preferencesService: any GemPreferencesServiceProtocol
    let walletId: WalletId
    let asset: Asset

    public let query: ObservableQuery<PriceAlertsRequest>
    public let priceQuery: ObservableQuery<PriceRequest>
    var priceAlerts: [PriceAlertData] {
        query.value
    }

    var isPresentingSetPriceAlert: Bool = false
    var isPresentingToastMessage: ToastMessage?

    public init(
        priceAlertService: any GemPriceAlertServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        walletId: WalletId,
        asset: Asset,
    ) {
        self.priceAlertService = priceAlertService
        self.preferencesService = preferencesService
        self.walletId = walletId
        self.asset = asset
        query = ObservableQuery(PriceAlertsRequest(assetId: asset.id), initialValue: [])
        priceQuery = ObservableQuery(PriceRequest(assetId: asset.id), initialValue: nil)
    }

    var title: String {
        Localized.Settings.PriceAlerts.title
    }

    var autoAlertItemModel: PriceAlertItemViewModel {
        PriceAlertItemViewModel(
            data: PriceAlertData(
                asset: asset,
                price: priceQuery.value?.price,
                priceAlert: .default(for: asset.id, currency: .default),
            ),
            currency: preferencesService.currencyCode,
        )
    }

    var isAutoAlertEnabledBinding: Binding<Bool> {
        Binding(
            get: { self.priceAlerts.contains(where: { $0.priceAlert.type == .auto }) },
            set: { newValue in
                Task { await self.toggleAutoAlert(enabled: newValue) }
            },
        )
    }

    var alertsModel: [PriceAlertItemViewModel] {
        priceAlerts
            .filter { $0.priceAlert.type != .auto }
            .displayedAlerts
            .map { PriceAlertItemViewModel(data: $0, currency: preferencesService.currencyCode) }
    }
}

// MARK: - Business Logic

extension AssetPriceAlertsViewModel {
    func load() async {
        do {
            try await priceAlertService.sync(assetId: asset.id.identifier)
        } catch {
            debugLog("load error: \(error)")
        }
    }

    func toggleAutoAlert(enabled: Bool) async {
        do {
            let currency = preferencesService.currency
            if enabled {
                try await priceAlertService.enable(priceAlert: .default(for: asset.id, currency: currency))
            } else {
                try await priceAlertService.delete(priceAlerts: [.default(for: asset.id, currency: currency)])
            }
        } catch {
            debugLog("toggleAutoAlert error: \(error)")
        }
    }

    func deletePriceAlert(priceAlert: PriceAlert) async {
        do {
            try await priceAlertService.delete(priceAlerts: [priceAlert])
        } catch {
            debugLog("deletePriceAlert error: \(error)")
        }
    }

    func onSelectSetPriceAlert() {
        isPresentingSetPriceAlert = true
    }

    func onSetPriceAlertComplete(message: String) {
        isPresentingSetPriceAlert = false
        isPresentingToastMessage = .priceAlert(message: message)
    }
}
