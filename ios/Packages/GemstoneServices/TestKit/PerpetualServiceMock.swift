// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualSocketUpdate
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

    public func getPositions(walletId _: WalletId, address _: String) async throws -> PerpetualAccountMode { .standard }
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualServiceMock: HyperliquidPerpetualServiceable {
    public func accountMode(walletId _: WalletId, address _: String) async -> PerpetualAccountMode {
        .standard
    }

    public func applySocketMessage(walletId _: WalletId, mode _: PerpetualAccountMode, data _: Data) async throws -> GemPerpetualSocketUpdate { .applied }
}
