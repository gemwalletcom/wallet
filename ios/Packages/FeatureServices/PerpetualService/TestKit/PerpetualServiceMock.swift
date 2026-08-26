// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PerpetualService
import Primitives

public struct PerpetualServiceMock: PerpetualServiceable {
    public init() {}

    public var marketsUpdatedAt: Date? { nil }

    public func updateMarkets() async throws {}

    public func clearMarkets() throws {}

    public func candlesticks(symbol _: String, period _: ChartPeriod) async throws -> [ChartCandleStick] {
        []
    }

    public func portfolio(address _: String) async throws -> PerpetualPortfolio {
        PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil)
    }

    public func setPinned(_: Bool, perpetualId _: PerpetualId) throws {}

    public func getPositions(walletId _: WalletId, address _: String) async throws {}
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualServiceMock: HyperliquidPerpetualServiceable {
    public func accountMode(walletId _: WalletId, address _: String) async -> PerpetualAccountMode {
        .standard
    }

    public func getHypercorePositions(walletId _: WalletId) throws -> [Primitives.PerpetualPosition] {
        []
    }

    public func updateBalance(walletId _: WalletId, balance _: Primitives.PerpetualBalance) throws {}

    public func diffPositions(deleteIds _: [String], positions _: [Primitives.PerpetualPosition], walletId _: WalletId) throws {}

    public func updateMarket(_: Primitives.PerpetualMarketData) throws {}

    public func updatePrices(_: [String: Double]) throws {}
}
