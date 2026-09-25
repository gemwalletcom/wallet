// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.AssetPrice
import struct Gemstone.GemChart
import enum Gemstone.GemChartPhase
import protocol Gemstone.GemChartServiceProtocol
import struct Gemstone.GemChartSession
import enum Gemstone.GemInfoTopic
import struct Gemstone.GemListSection
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
    private let preferences: ObservablePreferences

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
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    var title: String {
        assetModel.name
    }

    var asset: Asset {
        assetModel.asset
    }

    public var chartState: StateViewType<ChartValuesViewModel> {
        switch session.viewState(price: currentPrice).phase {
        case .loading: .loading
        case let .data(data):
            .data(ChartValuesViewModel(period: selectedPeriod, chartData: data))
        case .noData: .noData
        case let .failed(error): .error(error)
        }
    }

    private var currentPrice: AssetPrice? {
        priceData?.price.map {
            AssetPrice(
                assetId: asset.id.identifier,
                price: $0.price,
                priceChangePercentage24h: $0.priceChangePercentage24h,
                updatedAt: $0.updatedAt,
            )
        }
    }

    private(set) var sections: [GemListSection] = []

    public init(
        service: any GemChartServiceProtocol,
        preferences: ObservablePreferences,
        assetModel: AssetViewModel,
        onSetPriceAlert: @escaping (Asset) -> Void,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.service = service
        self.preferences = preferences
        self.assetModel = assetModel
        session = service.newSession()
        priceQuery = ObservableQuery(PriceRequest(assetId: assetModel.asset.id), initialValue: .with(asset: assetModel.asset))
        self.onSetPriceAlert = onSetPriceAlert
        self.onSelectAddress = onSelectAddress
    }
}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func load() async {
        let period = selectedPeriod.toGem()
        session = session.onRefresh()
        do {
            let chart = try await service.syncCharts(assetId: assetModel.asset.id.identifier, period: period)
            session = session.onLoaded(chart: chart, period: period)
        } catch let error as GemServiceError {
            session = session.onFailed(error: error, period: period)
        } catch {
            debugLog("chart scene: load error \(error)")
        }
    }

    func updateSections() async {
        guard let priceData else { return }
        do {
            let sections = try await service.sections(
                asset: priceData.asset.toGem(),
                price: priceData.price?.price,
                market: priceData.market?.toGem(),
                priceAlerts: priceData.priceAlerts.map { $0.toGem() },
                links: priceData.links.map { $0.toGem() },
            )
            guard !Task.isCancelled, priceData == self.priceData else { return }
            self.sections = sections
        } catch {
            debugLog("chart scene: sections error \(error)")
        }
    }

    var currency: Primitives.Currency {
        preferences.currency
    }

    func onChangeCurrency() async {
        let next = session.onCurrency(currency: currency.toGem())
        if next != session {
            session = next
            await load()
        }
        await updateSections()
    }

    func onSelectSetPriceAlerts() {
        onSetPriceAlert(assetModel.asset)
    }

    var onSelectContract: ((String) -> Void)? {
        guard let onSelectAddress else { return nil }
        let chain = asset.chain
        return { onSelectAddress(ChainAddress(chain: chain, address: $0)) }
    }

    internal func onInfo(_ topic: GemInfoTopic) {
        isPresentingInfoSheet = InfoSheetType(topic: topic, assetImage: nil)
    }
}
