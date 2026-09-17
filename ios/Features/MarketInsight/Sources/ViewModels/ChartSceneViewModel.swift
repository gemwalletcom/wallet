// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemAssetMarketRow
import struct Gemstone.GemChart
import enum Gemstone.GemChartPhase
import enum Gemstone.GemChartSection
import func Gemstone.socialLinks
import typealias Gemstone.AssetLink
import struct Gemstone.GemChartSession
import protocol Gemstone.GemChartServiceProtocol
import enum Gemstone.GemServiceError
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

    let walletId: WalletId
    let assetModel: AssetViewModel

    private var session: GemChartSession
    public var selectedPeriod: ChartPeriod {
        get { session.period.toPrimitives() }
        set {
            session = session.onSelectPeriod(period: newValue.toGem())
            do {
                try service.setChartPeriod(period: newValue.toGem())
            } catch {
                debugLog("ChartSceneViewModel chart period error: \(error)")
            }
        }
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
        switch session.viewState().phase {
        case .loading: .loading
        case let .data(data):
            ChartValuesViewModel(period: selectedPeriod, chartData: data)
                .map { .data($0) } ?? .noData
        case .noData: .noData
        case let .failed(error): .error(error)
        }
    }

    var sections: [GemChartSection] {
        guard let priceData else { return [] }
        return service.sections(
            asset: priceData.asset.toGem(),
            price: priceData.price?.price,
            market: priceData.market?.toGem(),
            priceAlerts: priceData.priceAlerts.map { $0.toGem() },
            links: priceData.links.map { $0.toGem() },
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
        session = service.newSession()
        priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: .with(asset: assetModel.asset))
        self.onSetPriceAlert = onSetPriceAlert
    }

    func socialLinksModel(_ links: [Gemstone.AssetLink]) -> SocialLinksViewModel {
        SocialLinksViewModel(links: socialLinks(links: links))
    }

    func listItem(for section: GemChartSection) -> ListItemModel {
        switch section {
        case let .priceAlerts(count): ListItemModel(title: section.title ?? "", subtitle: "\(count)")
        case .setPriceAlert, .market, .links: ListItemModel(title: section.title ?? "")
        }
    }

    func marketValues(_ rows: [GemAssetMarketRow]) -> [MarketValueViewModel] {
        AssetDetailsInfoViewModel(asset: asset, currency: service.currency).marketValues(rows)
    }

}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func load() async {
        session = session.onRefresh()
        do {
            session = try await session.onLoaded(chart: service.syncCharts(assetId: assetModel.asset.id.identifier, period: selectedPeriod.toGem()))
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
            session = session.onFailed(error: .Core(msg: error.localizedDescription))
        }
    }

    func onSelectSetPriceAlerts() {
        onSetPriceAlert(assetModel.asset)
    }

    internal func onSelectInfoSheet(_ type: InfoSheetType) {
        isPresentingInfoSheet = type
    }
}
