// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
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

    public init(
        wallet: Wallet,
        service: any GemPortfolioServiceProtocol,
        preferences: ObservablePreferences,
        defaultType: PortfolioType = .wallet,
    ) {
        self.wallet = wallet
        self.service = service
        self.preferences = preferences
        session = portfolioSession(portfolioType: defaultType.toGem())
    }

    private var state: GemPortfolioViewState {
        session.viewState()
    }

    var selectedType: PortfolioType {
        get { state.portfolioType.toPrimitives() }
        set { session = session.onSelectType(portfolioType: newValue.toGem()) }
    }

    var selectedChartType: PortfolioChartType {
        get { state.chartType.toPrimitives() }
        set { session = session.onSelectChartType(chartType: newValue.toGem()) }
    }

    public var selectedPeriod: ChartPeriod {
        get { state.period.toPrimitives() }
        set { session = session.onSelectPeriod(period: newValue.toGem()) }
    }

    public var periods: [ChartPeriod] {
        state.periods.map { $0.toPrimitives() }
    }

    public var chartState: StateViewType<ChartValuesViewModel> {
        switch state.phase {
        case .loading: .loading
        case let .data(chart): .data(ChartValuesViewModel(period: selectedPeriod, chartData: chart))
        case .noData: .noData
        case let .failed(error): .error(error)
        }
    }

    var showSegmentedControl: Bool {
        preferences.isPerpetualEnabled && service.showPerpetuals(walletType: wallet.type.toGem(), chains: wallet.chains.map(\.rawValue))
    }

    var navigationTitle: String {
        showSegmentedControl ? "" : typeTitle(for: selectedType)
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
        session = session.onSelectWallet(walletId: wallet.id.id, currency: preferences.currency.toGem())
        let result = await service.refresh(wallet: wallet.toGem(), request: session.request())
        session = session.onResult(result: result)
    }

    func loadIfNeeded() async {
        guard session.needsLoad() else { return }
        await load()
    }

    func typeTitle(for type: PortfolioType) -> String {
        type.title
    }

    func chartTypeTitle(for type: PortfolioChartType) -> String {
        type.title
    }
}
