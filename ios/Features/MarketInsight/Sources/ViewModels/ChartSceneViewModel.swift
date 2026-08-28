// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPreferencesServiceProtocol
import protocol Gemstone.GemPriceAlertServiceProtocol
import Components
import Formatters
import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemChartServiceProtocol
import GemstonePrimitives
import InfoSheet
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@MainActor
@Observable
public final class ChartSceneViewModel: ChartListViewable {
    private let service: any GemChartServiceProtocol
    private let priceStore: PriceStore
    private let preferencesService: any GemPreferencesServiceProtocol

    private var currencyCode: String {
        preferencesService.currencyCode
    }

    let walletId: WalletId
    let assetModel: AssetViewModel
    private let explorerService: any GemExplorerServiceProtocol
    let priceAlertService: any GemPriceAlertServiceProtocol

    public var chartState: StateViewType<ChartValuesViewModel> = .loading
    public var selectedPeriod: ChartPeriod {
        didSet { preferencesService.setChartPeriodValue(selectedPeriod) }
    }

    public let priceQuery: ObservableQuery<PriceRequest>
    var priceData: PriceData? {
        priceQuery.value
    }

    var isPresentingInfoSheet: InfoSheetType?
    private let onSetPriceAlert: (Asset) -> Void

    var title: String {
        assetModel.name
    }

    var priceAlertsViewModel: PriceAlertsViewModel {
        PriceAlertsViewModel(priceAlerts: priceData?.priceAlerts ?? [])
    }

    var showPriceAlerts: Bool {
        priceAlertsViewModel.hasPriceAlerts && isPriceAvailable
    }

    var isPriceAvailable: Bool {
        PriceViewModel(price: priceData?.price, currencyCode: currencyCode).isPriceAvailable
    }

    public init(
        explorerService: any GemExplorerServiceProtocol,
        service: any GemChartServiceProtocol,
        priceStore: PriceStore,
        assetModel: AssetViewModel,
        priceAlertService: any GemPriceAlertServiceProtocol,
        walletId: WalletId,
        preferencesService: any GemPreferencesServiceProtocol,
        onSetPriceAlert: @escaping (Asset) -> Void,
    ) {
        self.service = service
        self.priceStore = priceStore
        self.preferencesService = preferencesService
        self.assetModel = assetModel
        self.priceAlertService = priceAlertService
        self.explorerService = explorerService
        self.walletId = walletId
        selectedPeriod = preferencesService.chartPeriodValue
        priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: nil)
        self.onSetPriceAlert = onSetPriceAlert
    }

    var priceDataModel: AssetDetailsInfoViewModel? {
        guard let priceData else { return nil }
        return AssetDetailsInfoViewModel(explorerService: explorerService, priceData: priceData, currency: preferencesService.currencyCode)
    }
}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func load() async {
        chartState = .loading
        do {
            var charts = try await service.syncCharts(
                assetId: assetModel.asset.id.identifier,
                period: selectedPeriod.json(),
                currency: preferencesService.getCurrency(),
            ).map { try ChartDateValue($0) }
            let price = try priceStore.getPrices(for: [assetModel.asset.id.identifier]).first

            if let price, let last = charts.last, price.updatedAt > last.date {
                charts.append(ChartDateValue(date: .now, value: price.price))
            }

            let chartValues = try ChartValues.from(charts: charts)
            let formatter = CurrencyFormatter(currencyCode: currencyCode)
            let model = ChartValuesViewModel(
                period: selectedPeriod,
                price: price?.mapToPrice(),
                values: chartValues,
                formatter: formatter,
            )
            chartState = .data(model)
            if priceData?.priceAlerts.isNotEmpty == true {
                Task {
                    do {
                        try await priceAlertService.sync(assetId: assetModel.asset.id.identifier)
                    } catch {
                        debugLog("chart scene: price alerts update error \(error)")
                    }
                }
            }
        } catch {
            chartState.setError(error)
        }
    }

    func onSelectSetPriceAlerts() {
        onSetPriceAlert(assetModel.asset)
    }

    internal func onSelectInfoSheet(_ type: InfoSheetType) {
        isPresentingInfoSheet = type
    }
}
