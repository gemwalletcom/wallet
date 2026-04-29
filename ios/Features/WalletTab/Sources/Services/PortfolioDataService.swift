// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PerpetualService
import PriceService
import Primitives

enum PortfolioDataInput {
    case wallet(walletId: WalletId, period: ChartPeriod, currencyCode: String)
    case perpetuals(period: ChartPeriod, address: String)
}

public struct PortfolioDataService: Sendable {
    private let portfolioService: PortfolioService
    private let perpetualService: any PerpetualServiceable
    private let priceService: PriceService

    public init(
        portfolioService: PortfolioService,
        perpetualService: any PerpetualServiceable,
        priceService: PriceService,
    ) {
        self.portfolioService = portfolioService
        self.perpetualService = perpetualService
        self.priceService = priceService
    }

    func getPortfoliData(input: PortfolioDataInput) async throws -> PortfolioData {
        switch input {
        case let .wallet(walletId, period, currencyCode):
            try await getWalletData(walletId: walletId, period: period, currencyCode: currencyCode)
        case let .perpetuals(period, address):
            try await getPerpetualData(address: address, period: period)
        }
    }
}

// MARK: - Private

extension PortfolioDataService {
    private func getWalletData(walletId: WalletId, period: ChartPeriod, currencyCode: String) async throws -> PortfolioData {
        let rate = try priceService.getRate(currency: currencyCode)
        let portfolio = try await portfolioService.getPortfolioAssets(walletId: walletId, period: period)

        let values = portfolio.values.map {
            ChartDateValue(
                date: Date(timeIntervalSince1970: TimeInterval($0.timestamp)),
                value: Double($0.value) * rate,
            )
        }

        let charts = [PortfolioChartData(chartType: .value, values: values)]

        let statistics: [PortfolioStatistic] = [
            portfolio.allTimeHigh.map { .allTimeHigh($0) },
            portfolio.allTimeLow.map { .allTimeLow($0) },
        ].compactMap(\.self)

        return PortfolioData(charts: charts, statistics: statistics, availablePeriods: [.day, .week, .month, .year, .all])
    }

    private func getPerpetualData(address: String, period: ChartPeriod) async throws -> PortfolioData {
        let portfolio = try await perpetualService.portfolio(address: address)
        let timeframe = portfolio.timeframeData(for: period)

        let pnlValues = timeframe?.pnlHistory ?? []
        let valueValues = timeframe.map { Array($0.accountValueHistory.drop(while: { $0.value == .zero })) } ?? []

        let charts = [
            PortfolioChartData(chartType: .pnl, values: pnlValues),
            PortfolioChartData(chartType: .value, values: valueValues),
        ]

        let summaryStats: [PortfolioStatistic] = portfolio.accountSummary.map { summary in
            [
                .unrealizedPnl(summary.unrealizedPnl),
                .accountLeverage(summary.accountLeverage),
                .marginUsage(PortfolioMarginUsage(accountValue: summary.accountValue, usage: summary.marginUsage)),
            ]
        } ?? []

        let allTimeStats: [PortfolioStatistic] = portfolio.allTime.map { allTime in
            [
                allTime.pnlHistory.last.map { .allTimePnl($0.value) },
                .volume(allTime.volume),
            ].compactMap(\.self)
        } ?? []

        return PortfolioData(
            charts: charts,
            statistics: summaryStats + allTimeStats,
            availablePeriods: portfolio.availablePeriods,
        )
    }
}
