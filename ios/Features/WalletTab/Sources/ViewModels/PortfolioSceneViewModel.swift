// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Style

@Observable
@MainActor
public final class PortfolioSceneViewModel: ChartListViewable {
    private let wallet: Wallet
    private let service: PortfolioDataService
    private let preferences: ObservablePreferences

    private let currencyFormatter: CurrencyFormatter
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter: CurrencyFormatter
    private let perpetualFormatter = CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue)

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
        service: PortfolioDataService,
        preferences: ObservablePreferences,
        defaultType: PortfolioType = .wallet,
    ) {
        self.wallet = wallet
        self.service = service
        self.preferences = preferences
        let currencyCode = preferences.preferences.currency
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
        priceFormatter = CurrencyFormatter(currencyCode: currencyCode)
        percentFormatter = CurrencyFormatter(type: .percent, currencyCode: currencyCode)
        state = PortfolioState(selectedType: defaultType)
    }

    var showSegmentedControl: Bool {
        preferences.isPerpetualEnabled && wallet.isMultiCoins && wallet.hyperliquidAccount != nil
    }

    var navigationTitle: String {
        showSegmentedControl ? "" : typeTitle(for: state.selectedType)
    }

    public var chartState: StateViewType<ChartValuesViewModel> {
        switch selectedState {
        case .loading: .loading
        case .noData: .noData
        case let .error(error): .error(error)
        case let .data(data): chartViewModel(from: data).map { .data($0) } ?? .noData
        }
    }

    public var periods: [ChartPeriod] {
        selectedState.value?.availablePeriods ?? [.day, .week, .month, .year, .all]
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
    public func fetch() async {
        selectedState = .loading
        do {
            let data = try await service.getPortfoliData(input: getDataInput())
            if data.availablePeriods.isNotEmpty, !data.availablePeriods.contains(selectedPeriod) {
                selectedPeriod = data.availablePeriods.first ?? selectedPeriod
            }
            selectedState = .data(data)
        } catch {
            selectedState.setError(error)
        }
    }

    func onTypeChanged(_: PortfolioType, _: PortfolioType) {
        guard selectedState.value == nil else { return }
        Task { await fetch() }
    }

    func typeTitle(for type: PortfolioType) -> String {
        switch type {
        case .wallet: Localized.Wallet.Portfolio.title
        case .perpetuals: Localized.Perpetuals.title
        }
    }

    func chartTypeTitle(for type: PortfolioChartType) -> String {
        switch type {
        case .value: Localized.Perpetual.value
        case .pnl: Localized.Perpetual.pnl
        }
    }

    func statisticModel(_ statistic: PortfolioStatistic) -> ListItemModel {
        switch statistic {
        case let .allTimeHigh(chartValue):
            allTimeModel(title: Localized.Asset.allTimeHigh, chartValue: chartValue)
        case let .allTimeLow(chartValue):
            allTimeModel(title: Localized.Asset.allTimeLow, chartValue: chartValue)
        case let .unrealizedPnl(value):
            pnlModel(title: Localized.Perpetual.unrealizedPnl, value: value)
        case let .accountLeverage(value):
            ListItemModel(
                title: Localized.Perpetual.accountLeverage,
                subtitle: value.formatted(.number.precision(.fractionLength(2))) + "x",
            )
        case let .marginUsage(margin):
            marginModel(margin)
        case let .allTimePnl(value):
            pnlModel(title: Localized.Perpetual.allTimePnl, value: value)
        case let .volume(value):
            ListItemModel(title: Localized.Perpetual.volume, subtitle: perpetualFormatter.string(value))
        }
    }
}

// MARK: - Private

extension PortfolioSceneViewModel {
    private func getDataInput() throws -> PortfolioDataInput {
        switch state.selectedType {
        case .wallet:
            return .wallet(walletId: wallet.walletId, period: selectedPeriod, currencyCode: currencyCode)
        case .perpetuals:
            guard let address = wallet.hyperliquidAccount?.address else {
                throw AnyError("perpetual account not available")
            }
            return .perpetuals(period: selectedPeriod, address: address)
        }
    }

    private func chartViewModel(from data: PortfolioData) -> ChartValuesViewModel? {
        let charts = data.charts.first(where: { $0.chartType == state.selectedChartType })?.values
            ?? data.charts.first?.values
            ?? []
        return .priceChange(
            charts: charts,
            period: selectedPeriod,
            formatter: chartFormatter,
            showHeaderValue: state.selectedType == .wallet || state.selectedChartType == .value,
        )
    }

    private func allTimeModel(title: String, chartValue: ChartValuePercentage) -> ListItemModel {
        AllTimeValueViewModel(priceFormatter: priceFormatter, percentFormatter: percentFormatter)
            .model(title: title, chartValue: chartValue)
    }

    private func pnlModel(title: String, value: Double) -> ListItemModel {
        let pnl = PriceChangeViewModel(value: value, currencyFormatter: perpetualFormatter)
        return ListItemModel(title: title, subtitle: pnl.text ?? "-", subtitleStyle: pnl.textStyle)
    }

    private func marginModel(_ margin: PortfolioMarginUsage) -> ListItemModel {
        let value = perpetualFormatter.string(margin.accountValue * margin.usage)
        let percent = CurrencyFormatter.percentSignLess.string(margin.usage * 100)
        return ListItemModel(title: Localized.Perpetual.marginUsage, subtitle: "\(value) (\(percent))")
    }

    private var currencyCode: String {
        preferences.preferences.currency
    }

    private var chartFormatter: CurrencyFormatter {
        state.selectedType == .perpetuals ? perpetualFormatter : currencyFormatter
    }
}
