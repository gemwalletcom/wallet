// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemChartData
import struct Gemstone.GemChartInput
import enum Gemstone.GemChartPhase
import protocol Gemstone.GemChartServiceProtocol
import struct Gemstone.GemChartSession
import struct Gemstone.GemChartView
import struct Gemstone.GemInfoSheet
import enum Gemstone.GemInfoTopic
import struct Gemstone.GemListSection
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Primitives
import PrimitivesComponents
import Store
import SwiftUI

@MainActor
@Observable
public final class ChartSceneViewModel: ChartListViewable {
    private let service: any GemChartServiceProtocol
    private let preferences: ObservablePreferences

    let asset: Asset

    private var session: GemChartSession {
        didSet { view = service.viewState(session: session, input: priceData.map()) }
    }

    private(set) var view: GemChartView

    public var selectedPeriod: ChartPeriod {
        get { session.period.toPrimitives() }
        set { session = session.onSelectPeriod(period: newValue.toGem()) }
    }

    public let priceQuery: ObservableQuery<PriceQuery>
    var priceData: PriceData {
        priceQuery.value ?? .with(asset: asset)
    }

    var isPresentingInfoSheet: GemInfoSheet?
    private let onSetPriceAlert: (Asset) -> Void
    private let onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)?

    var title: String {
        asset.name
    }

    public var chartState: StateViewType<GemChartData> {
        view.phase.map()
    }

    var sections: [GemListSection] {
        view.sections
    }

    public init(
        service: any GemChartServiceProtocol,
        preferences: ObservablePreferences,
        asset: Asset,
        onSetPriceAlert: @escaping (Asset) -> Void,
        onSelectAddress: (@MainActor @Sendable (ChainAddress) -> Void)? = nil,
    ) {
        self.service = service
        self.preferences = preferences
        self.asset = asset
        let session = service.newSession()
        let priceData = PriceData.with(asset: asset)
        self.session = session
        priceQuery = ObservableQuery(PriceQuery(assetId: asset.id), initialValue: priceData)
        view = service.viewState(session: session, input: priceData.map())
        self.onSetPriceAlert = onSetPriceAlert
        self.onSelectAddress = onSelectAddress
    }
}

// MARK: - Business Logic

public extension ChartSceneViewModel {
    func load() async {
        session = session.onRefresh()
        while !Task.isCancelled, let request = session.request() {
            let result = await service.load(assetId: asset.id, request: request)
            session = session.onResult(result: result)
        }
    }

    func onChangePriceData() {
        view = service.viewState(session: session, input: priceData.map())
    }

    func onZoom(_ magnification: Double, anchor: Double) {
        let zoomed = session.onZoom(magnification: magnification, anchor: anchor)
        guard zoomed.zoom != session.zoom else { return }
        session = zoomed
    }

    func onPan(_ fraction: Double) {
        let panned = session.onPan(fraction: fraction)
        guard panned.zoom != session.zoom else { return }
        session = panned
    }

    var currency: Primitives.Currency {
        preferences.currency
    }

    func onChangeCurrency() async {
        let next = session.onCurrency(currency: currency.toGem())
        guard next != session else { return }
        session = next
        await load()
    }

    func onSelectSetPriceAlerts() {
        onSetPriceAlert(asset)
    }

    var onSelectContract: ((String) -> Void)? {
        guard let onSelectAddress else { return nil }
        let chain = asset.chain
        return { onSelectAddress(ChainAddress(chain: chain, address: $0)) }
    }

    internal func onInfo(_ topic: GemInfoTopic) {
        isPresentingInfoSheet = topic.infoSheet
    }
}

// MARK: - Private

private extension PriceData {
    func map() -> GemChartInput {
        GemChartInput(
            asset: asset.toGem(),
            price: price?.toGem(),
            market: market?.toGem(),
            priceAlerts: priceAlerts.map { $0.toGem() },
            links: links.map { $0.toGem() },
        )
    }
}

private extension GemChartPhase {
    func map() -> StateViewType<GemChartData> {
        switch self {
        case .loading: .loading
        case let .data(data): .data(data)
        case .noData: .noData
        case let .failed(error): .error(error)
        }
    }
}
