// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemAssetMarketRow
import struct Gemstone.GemChart
import enum Gemstone.GemChartSection
import protocol Gemstone.GemChartServiceProtocol
import func Gemstone.priceChartData
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
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

    private var chart: StateViewType<GemChart> = .loading
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

    var asset: Asset {
        assetModel.asset
    }

    public var chartState: StateViewType<ChartValuesViewModel> {
        chart.flatMap { chartValuesViewModel(from: $0).map { .data($0) } ?? .noData }
    }

    var sections: [GemChartSection] {
        guard let priceData else { return [] }
        return service.sections(
            asset: priceData.asset.map(),
            price: priceData.price?.price,
            market: priceData.market?.map(),
            priceAlerts: priceData.priceAlerts.map { $0.map() },
            links: priceData.links.map { $0.map() },
        )
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
        priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: .with(asset: assetModel.asset))
        self.onSetPriceAlert = onSetPriceAlert
    }

    func marketValues(_ rows: [GemAssetMarketRow]) -> [MarketValueViewModel] {
        AssetDetailsInfoViewModel(asset: asset, currency: currencyCode).marketValues(rows)
    }

    private func chartValuesViewModel(from chart: GemChart) -> ChartValuesViewModel? {
        guard let chartData = priceChartData(chart: chart) else {
            return nil
        }
        return ChartValuesViewModel(period: selectedPeriod, chartData: chartData, formatter: CurrencyFormatter(currencyCode: currencyCode))
    }
}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func load() async {
        if chart.value == nil {
            chart = .loading
        }
        do {
            chart = try await .data(service.syncCharts(assetId: assetModel.asset.id.identifier, period: selectedPeriod.map()))
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
            chart.setError(error)
        }
    }

    func onSelectSetPriceAlerts() {
        onSetPriceAlert(assetModel.asset)
    }

    internal func onSelectInfoSheet(_ type: InfoSheetType) {
        isPresentingInfoSheet = type
    }
}
