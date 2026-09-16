// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import protocol Gemstone.GemPortfolioServiceProtocol
import func Gemstone.portfolioChartData
import struct Gemstone.PortfolioData
import struct Gemstone.PortfolioMarginUsage
import enum Gemstone.PortfolioStatistic
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
        get { state[state.selectedType] }
        set { state[state.selectedType] = newValue }
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

    var statistics: [PortfolioStatistic] {
        selectedState.value?.statistics ?? []
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
        selectedState = .loading
        do {
            let data = try await service.portfolioData(wallet: wallet.toGem(), portfolioType: state.selectedType.toGem(), period: selectedPeriod.toGem())
            let periods = data.availablePeriods.map { $0.toPrimitives() }
            if periods.isNotEmpty, !periods.contains(selectedPeriod) {
                selectedPeriod = periods.first ?? selectedPeriod
            }
            selectedState = .data(data)
        } catch {
            selectedState.setError(error)
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
        switch type {
        case .value: Localized.Perpetual.value
        case .pnl: Localized.Perpetual.pnl
        }
    }

    func statisticModel(_ statistic: PortfolioStatistic) -> ListItemModel {
        let title = statistic.title
        switch statistic {
        case let .allTimeHigh(chartValue), let .allTimeLow(chartValue):
            return allTimeModel(title: title, chartValue: chartValue.toPrimitives())
        case let .unrealizedPnl(value), let .allTimePnl(value):
            return pnlModel(title: title, value: value)
        case let .accountLeverage(value):
            return ListItemModel(title: title, subtitle: value.formatted(.number.precision(.fractionLength(2))) + "x")
        case let .marginUsage(margin):
            return marginModel(title: title, margin)
        case let .volume(value):
            return ListItemModel(title: title, subtitle: perpetualFormatter.string(value))
        }
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

    private func allTimeModel(title: String, chartValue: ChartValuePercentage) -> ListItemModel {
        AllTimeValueViewModel(priceFormatter: priceFormatter, percentFormatter: percentFormatter)
            .model(title: title, chartValue: chartValue)
    }

    private func pnlModel(title: String, value: Double) -> ListItemModel {
        let pnl = PriceChangeViewModel(value: value, currencyFormatter: perpetualFormatter)
        return ListItemModel(title: title, subtitle: pnl.text ?? "-", subtitleStyle: pnl.textStyle)
    }

    private func marginModel(title: String, _ margin: PortfolioMarginUsage) -> ListItemModel {
        let value = perpetualFormatter.string(margin.usedValue)
        let percent = PercentFormatter.unsigned.string(margin.usagePercent)
        return ListItemModel(title: title, subtitle: "\(value) (\(percent))")
    }
}
