// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemListRow
import protocol Gemstone.GemPortfolioServiceProtocol
import func Gemstone.leverageNumber
import func Gemstone.portfolioChartData
import struct Gemstone.PortfolioData
import struct Gemstone.PortfolioMarginUsage
import func Gemstone.portfolioStatisticRows
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class PortfolioSceneViewModel: ChartListViewable {
    private let wallet: Wallet
    private let service: any GemPortfolioServiceProtocol
    private let preferences: ObservablePreferences

    private let currencyFormatter: CurrencyFormatter
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter = PercentFormatter.signed
    private let perpetualFormatter: CurrencyFormatter

    var state: PortfolioState

    private var selectedState: StateViewType<PortfolioData> {
        state[state.selectedType]
    }

    public var selectedPeriod: ChartPeriod {
        get { state.selectedPeriod }
        set { state.selectedPeriod = newValue }
    }

    public init(
        wallet: Wallet,
        service: any GemPortfolioServiceProtocol,
        preferences: ObservablePreferences,
        defaultType: PortfolioType = .wallet,
    ) {
        self.wallet = wallet
        self.service = service
        self.preferences = preferences
        perpetualFormatter = CurrencyFormatter(type: .currency, currencyCode: service.currency(portfolioType: PortfolioType.perpetuals.toGem()).toPrimitives().rawValue)
        let currencyCode = preferences.currency.rawValue
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
        priceFormatter = CurrencyFormatter(currencyCode: currencyCode)
        state = PortfolioState(selectedType: defaultType)
    }

    var showSegmentedControl: Bool {
        preferences.showPerpetuals(for: wallet)
    }

    var navigationTitle: String {
        showSegmentedControl ? "" : typeTitle(for: state.selectedType)
    }

    public var chartState: StateViewType<ChartValuesViewModel> {
        selectedState.flatMap { chartViewModel(from: $0).map { .data($0) } ?? .noData }
    }

    public var periods: [ChartPeriod] {
        selectedState.value?.availablePeriods.map { $0.toPrimitives() } ?? [.day, .week, .month, .year, .all]
    }

    var statisticRows: [GemListRow] {
        portfolioStatisticRows(
            statistics: selectedState.value?.statistics ?? [],
            currency: service.currency(portfolioType: state.selectedType.toGem()),
        )
    }

    var statisticsTitle: String {
        Localized.Common.info
    }

    var showChartTypePicker: Bool {
        state.selectedType == .perpetuals
    }
}

// MARK: - Business Logic

extension PortfolioSceneViewModel {
    public func load() async {
        let type = state.selectedType
        let period = selectedPeriod
        state[type] = .loading
        do {
            let data = try await service.portfolioData(wallet: wallet.toGem(), portfolioType: type.toGem(), period: period.toGem())
            guard period == selectedPeriod else { return }
            let periods = data.availablePeriods.map { $0.toPrimitives() }
            if periods.isNotEmpty, !periods.contains(period) {
                selectedPeriod = periods.first ?? period
            }
            state[type] = .data(data)
        } catch {
            guard period == selectedPeriod else { return }
            state[type].setError(error)
        }
    }

    func onTypeChanged(_: PortfolioType, _: PortfolioType) {
        guard selectedState.value == nil else { return }
        Task { await load() }
    }

    func typeTitle(for type: PortfolioType) -> String {
        type.title
    }

    func chartTypeTitle(for type: PortfolioChartType) -> String {
        type.title
    }
}

// MARK: - Private

extension PortfolioSceneViewModel {
    private func chartViewModel(from data: PortfolioData) -> ChartValuesViewModel? {
        let portfolioType = state.selectedType.toGem()
        guard let chartData = portfolioChartData(
            data: data,
            portfolioType: portfolioType,
            chartType: state.selectedChartType.toGem(),
            currency: service.currency(portfolioType: portfolioType),
        ) else {
            return nil
        }
        return ChartValuesViewModel(period: selectedPeriod, chartData: chartData)
    }
}
