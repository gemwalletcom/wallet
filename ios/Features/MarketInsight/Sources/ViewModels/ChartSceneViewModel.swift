// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceServiceProtocol
import Components
import Formatters
import Foundation
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemChartServiceProtocol
import GemstonePrimitives
import InfoSheet
import Localization
import Preferences
import GemstoneServices
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@MainActor
@Observable
public final class ChartSceneViewModel: ChartListViewable {
    private let service: any GemChartServiceProtocol
    private let priceService: any GemPriceServiceProtocol
    private let priceStore: PriceStore
    private let preferences: Preferences

    let walletId: WalletId
    let assetModel: AssetViewModel
    private let explorerService: any GemExplorerServiceProtocol
    let priceAlertService: PriceAlertService

    public var chartState: StateViewType<ChartValuesViewModel> = .loading
    public var selectedPeriod: ChartPeriod {
        didSet { preferences.chartPeriod = selectedPeriod }
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
        PriceViewModel(price: priceData?.price, currencyCode: preferences.currency).isPriceAvailable
    }

    public init(
        explorerService: any GemExplorerServiceProtocol,
        service: any GemChartServiceProtocol,
        priceService: any GemPriceServiceProtocol,
        priceStore: PriceStore,
        assetModel: AssetViewModel,
        priceAlertService: PriceAlertService,
        walletId: WalletId,
        preferences: Preferences = .standard,
        onSetPriceAlert: @escaping (Asset) -> Void,
    ) {
        self.service = service
        self.priceService = priceService
        self.priceStore = priceStore
        self.preferences = preferences
        self.assetModel = assetModel
        self.priceAlertService = priceAlertService
        self.explorerService = explorerService
        self.walletId = walletId
        selectedPeriod = preferences.chartPeriod
        priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: nil)
        self.onSetPriceAlert = onSetPriceAlert
    }

    var priceDataModel: AssetDetailsInfoViewModel? {
        guard let priceData else { return nil }
        return AssetDetailsInfoViewModel(explorerService: explorerService, priceData: priceData)
    }
}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func fetch() async {
        chartState = .loading
        do {
            let values = try await Primitives.Charts(service.getCharts(assetId: assetModel.asset.id.identifier, period: selectedPeriod.json()))
            if let market = values.market {
                try await priceService.updateMarket(assetId: assetModel.asset.id.identifier, market: market.json(), currency: Currency(id: preferences.currency).json())
            }
            let price = try priceStore.getPrices(for: [assetModel.asset.id.identifier]).first
            let rate = try priceStore.getRate(currency: preferences.currency).rate

            var charts = values.prices.map {
                ChartDateValue(date: Date(timeIntervalSince1970: TimeInterval($0.timestamp)), value: Double($0.value) * rate)
            }

            if let price, let last = charts.last, price.updatedAt > last.date {
                charts.append(ChartDateValue(date: .now, value: price.price))
            }

            let chartValues = try ChartValues.from(charts: charts)
            let formatter = CurrencyFormatter(currencyCode: preferences.currency)
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
                        try await priceAlertService.update(assetId: assetModel.asset.id.identifier)
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
