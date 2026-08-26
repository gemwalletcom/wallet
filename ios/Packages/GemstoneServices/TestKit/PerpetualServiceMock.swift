// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Primitives

public struct PerpetualServiceMock: PerpetualServiceable {
    public init() {}

    public var marketsUpdatedAt: Date? { nil }

    public func updateMarkets() async throws {}

    public func clearMarkets() async throws {}

    public func candlesticks(symbol _: String, period _: ChartPeriod) async throws -> [ChartCandleStick] {
        []
    }

    public func portfolio(address _: String) async throws -> PerpetualPortfolio {
        PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil)
    }

    public func setPinned(_: Bool, perpetualId _: PerpetualId) async throws {}

    public func getPositions(walletId _: WalletId, address _: String) async throws {}
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualServiceMock: HyperliquidPerpetualServiceable {
    public func accountMode(walletId _: WalletId, address _: String) async -> PerpetualAccountMode {
        .standard
    }

    public func getHypercorePositions(walletId _: WalletId) async throws -> [Primitives.PerpetualPosition] {
        []
    }

    public func updateBalance(walletId _: WalletId, balance _: Primitives.PerpetualBalance) async throws {}

    public func updatePositions(walletId _: WalletId, positions _: [Primitives.PerpetualPosition], deleteIds _: [String]) async throws {}

    public func updateMarket(_: Primitives.PerpetualMarketData) async throws {}

    public func updatePrices(_: [String: Double]) async throws {}
}
