// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import enum Gemstone.GemLoadState
import struct Gemstone.GemPriceAlertItem
import protocol Gemstone.GemPriceAlertServiceProtocol
import enum Gemstone.GemRowAction
import enum Gemstone.GemServiceError
import func Gemstone.loadError
import class Gemstone.PriceAlertFormatter
import func Gemstone.priceAlertsToggleRow
import GemstonePrimitives
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

    public let query: ObservableQuery<PriceAlertsQuery>
    var priceAlerts: [PriceAlertData] {
        query.value
    }

    var isPriceAlertsEnabled: Bool
    var isPresentingAlertMessage: AlertMessage?

    private var loadState: GemLoadState = .loading

    public init(
        service: any GemPriceAlertServiceProtocol,
    ) {
        self.service = service
        isPriceAlertsEnabled = service.isEnabled()
        query = ObservableQuery(PriceAlertsQuery(), initialValue: [])
    }

    var title: String {
        Localized.Settings.PriceAlerts.title
    }

    var currency: Currency {
        service.getCurrency().toPrimitives()
    }

    var toggleRow: GemListRow {
        priceAlertsToggleRow(enabled: isPriceAlertsEnabled)
    }

    var loadError: Error? {
        Gemstone.loadError(state: loadState, hasRows: !priceAlerts.isEmpty)
    }

    var emptyContentModel: EmptyStateViewModel {
        EmptyStateViewModel(kind: .priceAlerts)
    }

    func chart(_ item: GemPriceAlertItem) -> Scenes.Chart {
        Scenes.Chart(asset: item.data.asset.toPrimitives())
    }

    var sections: [ListItemValueSection<GemPriceAlertItem>] {
        PriceAlertFormatter.shared.sections(alerts: priceAlerts.map { $0.toGem() }, priceCurrency: currency.toGem()).map { section in
            ListItemValueSection(
                section: section.kind.title,
                footer: section.kind.footer,
                values: section.items.map { ListItemValue(value: $0) },
            )
        }
    }
}

// MARK: - Business Logic

extension PriceAlertsSceneViewModel {
    public func load() async {
        loadState = await service.refresh(assetId: nil, hasAlerts: priceAlerts.isNotEmpty)
    }

    func deletePriceAlert(priceAlert: PriceAlert) async {
        do {
            try await service.delete(priceAlerts: [priceAlert])
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
    }

    public func includeAsset(_ asset: Asset) async -> ToastMessage? {
        do {
            return try await ToastMessage(toast: service.setAutoAlert(asset: asset.toGem(), enabled: true))
        } catch let error as GemServiceError {
            return .error(error.text().text)
        } catch {
            debugLog("price alerts include asset error: \(error)")
            return nil
        }
    }

    func onToggle(_: GemRowAction, _ isOn: Bool) {
        isPriceAlertsEnabled = isOn
    }

    func setAlertsEnabled(_ enabled: Bool) async {
        do {
            try await service.setEnabled(enabled: enabled)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
        isPriceAlertsEnabled = service.isEnabled()
    }
}
