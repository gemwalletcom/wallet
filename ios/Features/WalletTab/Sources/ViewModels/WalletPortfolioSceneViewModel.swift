// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import PriceService
import Primitives
import PrimitivesComponents

@Observable
@MainActor
public final class WalletPortfolioSceneViewModel: ChartListViewable {
    private let wallet: Wallet

    private let service: PortfolioService
    private let priceService: PriceService

    private let currencyCode: String
    private let currencyFormatter: CurrencyFormatter
    private let priceFormatter: CurrencyFormatter
    private let percentFormatter: CurrencyFormatter

    var state: StateViewType<WalletPortfolioData> = .loading
    public var selectedPeriod: ChartPeriod = .day

    public init(
        wallet: Wallet,
        portfolioService: PortfolioService,
        priceService: PriceService,
        currencyCode: String,
    ) {
        service = portfolioService
        self.priceService = priceService
        self.wallet = wallet

        self.currencyCode = currencyCode
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: currencyCode)
        priceFormatter = CurrencyFormatter(currencyCode: currencyCode)
        percentFormatter = CurrencyFormatter(type: .percent, currencyCode: currencyCode)
    }

    var navigationTitle: String { Localized.Wallet.Portfolio.title }

    public var periods: [ChartPeriod] { [.day, .week, .month, .year, .all] }
    public var chartState: StateViewType<ChartValuesViewModel> {
        state.map(\.chart)
    }

    var allTimeValues: [ListItemModel] {
        guard case let .data(data) = state else { return [] }
        let allTime = AllTimeValueViewModel(priceFormatter: priceFormatter, percentFormatter: percentFormatter)
        return [
            data.portfolio.allTimeHigh.map { allTime.allTimeHigh(chartValue: $0) },
            data.portfolio.allTimeLow.map { allTime.allTimeLow(chartValue: $0) },
        ].compactMap(\.self)
    }
}

// MARK: - Business Logic

public extension WalletPortfolioSceneViewModel {
    func fetch() async {
        state = .loading
        do {
            let rate = try priceService.getRate(currency: currencyCode)
            let portfolio = try await service.getPortfolioAssets(walletId: wallet.walletId, period: selectedPeriod)
            let charts = portfolio.values.map {
                ChartDateValue(date: Date(timeIntervalSince1970: TimeInterval($0.timestamp)), value: Double($0.value) * rate)
            }
            let chart = ChartValuesViewModel.priceChange(charts: charts, period: selectedPeriod, formatter: currencyFormatter, showHeaderValue: true)
            state = chart.map { .data(WalletPortfolioData(chart: $0, portfolio: portfolio)) } ?? .noData
        } catch {
            state.setError(error)
        }
    }
}
