// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemChartData
import enum Gemstone.GemListRow
import protocol Gemstone.GemPortfolioServiceProtocol
import struct Gemstone.GemPortfolioSession
import struct Gemstone.GemPortfolioViewState
import func Gemstone.portfolioSession
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class PortfolioSceneViewModel: ChartListViewable {
    private let wallet: Wallet
    private let service: any GemPortfolioServiceProtocol
    private let preferences: ObservablePreferences

    private var session: GemPortfolioSession
    private var state: GemPortfolioViewState

    public init(
        wallet: Wallet,
        service: any GemPortfolioServiceProtocol,
        preferences: ObservablePreferences,
        defaultType: PortfolioType = .wallet,
    ) {
        self.wallet = wallet
        self.service = service
        self.preferences = preferences
        let session = portfolioSession(portfolioType: defaultType.toGem())
        self.session = session
        state = session.viewState()
    }

    var selectedType: PortfolioType {
        get { state.portfolioType.toPrimitives() }
        set { update(session.onSelectType(portfolioType: newValue.toGem())) }
    }

    var selectedChartType: PortfolioChartType {
        get { state.chartType.toPrimitives() }
        set { update(session.onSelectChartType(chartType: newValue.toGem())) }
    }

    public var selectedPeriod: ChartPeriod {
        get { state.period.toPrimitives() }
        set { update(session.onSelectPeriod(period: newValue.toGem())) }
    }

    public var periods: [ChartPeriod] {
        state.periods.map { $0.toPrimitives() }
    }

    public var chartState: StateViewType<GemChartData> {
        switch state.phase {
        case .loading: .loading
        case let .data(chart): .data(chart)
        case .noData: .noData
        case let .failed(error): .error(error)
        }
    }

    var showSegmentedControl: Bool {
        service.showPerpetuals(walletType: wallet.type.toGem(), chains: wallet.chains.map(\.rawValue))
    }

    var navigationTitle: String {
        showSegmentedControl ? "" : state.portfolioType.toPrimitives().title
    }

    var statisticRows: [GemListRow] {
        state.statistics
    }

    var statisticsTitle: String {
        Localized.Common.info
    }

    var showChartTypePicker: Bool {
        state.showsChartTypePicker
    }
}

// MARK: - Business Logic

extension PortfolioSceneViewModel {
    public func load() async {
        update(session.onSelectWallet(walletId: wallet.id.id, currency: preferences.currency.toGem()))
        let result = await service.refresh(wallet: wallet.toGem(), request: session.request())
        update(session.onResult(result: result))
    }

    func loadIfNeeded() async {
        guard session.needsLoad() else { return }
        await load()
    }

    private func update(_ session: GemPortfolioSession) {
        self.session = session
        state = session.viewState()
    }
}
