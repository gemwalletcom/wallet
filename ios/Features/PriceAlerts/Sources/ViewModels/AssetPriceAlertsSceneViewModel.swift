// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAssetPriceAlerts
import enum Gemstone.GemLoadState
import protocol Gemstone.GemPriceAlertServiceProtocol
import enum Gemstone.GemServiceError
import func Gemstone.loadError
import class Gemstone.PriceAlertFormatter
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@Observable
@MainActor
public final class AssetPriceAlertsSceneViewModel: Sendable {
    private let service: any GemPriceAlertServiceProtocol
    let walletId: WalletId
    let asset: Asset

    public let query: ObservableQuery<PriceAlertsQuery>
    public let priceQuery: ObservableQuery<PriceQuery>
    var priceAlerts: [PriceAlertData] {
        query.value
    }

    var isPresentingSetPriceAlert: Bool = false
    var isPresentingToastMessage: ToastMessage?

    private var loadState: GemLoadState = .loading

    var loadError: Error? {
        Gemstone.loadError(state: loadState, hasRows: !priceAlerts.isEmpty)
    }

    public init(
        service: any GemPriceAlertServiceProtocol,
        walletId: WalletId,
        asset: Asset,
    ) {
        self.service = service
        self.walletId = walletId
        self.asset = asset
        query = ObservableQuery(PriceAlertsQuery(assetId: asset.id), initialValue: [])
        priceQuery = ObservableQuery(PriceQuery(assetId: asset.id), initialValue: nil)
    }

    var title: String {
        Localized.Settings.PriceAlerts.title
    }

    func isAutoAlertEnabledBinding(_ assetAlerts: GemAssetPriceAlerts) -> Binding<Bool> {
        Binding(
            get: { assetAlerts.autoAlert == .enabled },
            set: { newValue in
                Task { await self.toggleAutoAlert(enabled: newValue) }
            },
        )
    }

    func alerts(_ assetAlerts: GemAssetPriceAlerts) -> [PriceAlertItemViewModel] {
        assetAlerts.alerts.map(PriceAlertItemViewModel.init(item:))
    }

    func showsEmpty(_ assetAlerts: GemAssetPriceAlerts) -> Bool {
        assetAlerts.showsEmpty && loadError == nil
    }

    var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .priceAlerts)
    }

    var assetAlerts: GemAssetPriceAlerts {
        PriceAlertFormatter.shared.assetAlerts(
            asset: asset.toGem(),
            price: priceQuery.value?.price?.toGem(),
            alerts: priceAlerts.map { $0.toGem() },
            priceCurrency: service.getCurrency(),
        )
    }
}

// MARK: - Business Logic

extension AssetPriceAlertsSceneViewModel {
    func load() async {
        loadState = await service.refresh(assetId: asset.id.identifier, hasAlerts: priceAlerts.isNotEmpty)
    }

    func toggleAutoAlert(enabled: Bool) async {
        do {
            try await service.setAutoAlert(assetId: asset.id.identifier, enabled: enabled)
        } catch let error as GemServiceError {
            isPresentingToastMessage = .error(error.text().text)
        } catch {
            debugLog("price alerts error: \(error)")
        }
    }

    func deletePriceAlert(priceAlert: PriceAlert) async {
        do {
            try await service.delete(priceAlerts: [priceAlert])
        } catch let error as GemServiceError {
            isPresentingToastMessage = .error(error.text().text)
        } catch {
            debugLog("price alerts error: \(error)")
        }
    }

    func onSelectSetPriceAlert() {
        isPresentingSetPriceAlert = true
    }

    func onSetPriceAlertComplete(message: String) {
        isPresentingSetPriceAlert = false
        isPresentingToastMessage = .priceAlert(message: message)
    }

    func setPriceAlertModel() -> SetPriceAlertSceneViewModel {
        SetPriceAlertSceneViewModel(walletId: walletId, asset: asset, service: service) { [weak self] in self?.onSetPriceAlertComplete(message: $0) }
    }
}
