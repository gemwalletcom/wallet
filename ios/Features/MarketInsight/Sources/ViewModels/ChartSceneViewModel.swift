// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
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

    private var currencyCode: String {
        service.getCurrency()
    }

    let walletId: WalletId
    let assetModel: AssetViewModel

    public var chartState: StateViewType<ChartValuesViewModel> = .loading
    public var selectedPeriod: ChartPeriod {
        didSet { try? service.setChartPeriod(period: selectedPeriod.map()) }
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
        service: any GemChartServiceProtocol,
        assetModel: AssetViewModel,
        walletId: WalletId,
        onSetPriceAlert: @escaping (Asset) -> Void,
    ) {
        self.service = service
        self.assetModel = assetModel
        self.walletId = walletId
        selectedPeriod = service.chartPeriod().map()
        priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: nil)
        self.onSetPriceAlert = onSetPriceAlert
    }

    var priceDataModel: AssetDetailsInfoViewModel? {
        guard let priceData else { return nil }
        return AssetDetailsInfoViewModel(
            priceData: priceData,
            currency: currencyCode,
            contractExplorerLink: (try? priceData.asset.getTokenId()).flatMap {
                service.tokenUrl(chain: priceData.asset.chain.rawValue, address: $0).map { $0.map() }
            },
        )
    }
}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func load() async {
        chartState = .loading
        do {
            let chart = try await service.syncCharts(assetId: assetModel.asset.id.identifier, period: selectedPeriod.map())
            let charts = (chart.values + [chart.current].compactMap { $0 }).map { $0.map() }
            let chartValues = try ChartValues.from(charts: charts)
            let formatter = CurrencyFormatter(currencyCode: currencyCode)
            let model = ChartValuesViewModel(
                period: selectedPeriod,
                price: priceData?.price,
                values: chartValues,
                formatter: formatter,
            )
            chartState = .data(model)
            if priceData?.priceAlerts.isNotEmpty == true {
                Task {
                    do {
                        try await service.syncPriceAlerts(assetId: assetModel.asset.id.identifier)
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
