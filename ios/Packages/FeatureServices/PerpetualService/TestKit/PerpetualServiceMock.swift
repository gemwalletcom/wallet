// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import PerpetualService
import Primitives

public struct PerpetualServiceMock: PerpetualServiceable {
    public init() {}

    public func updateMarkets() async throws {}

    public func candlesticks(symbol _: String, period _: ChartPeriod) async throws -> [ChartCandleStick] {
        []
    }

    public func portfolio(address _: String) async throws -> PerpetualPortfolio {
        PerpetualPortfolio(day: nil, week: nil, month: nil, allTime: nil, accountSummary: nil)
    }

    public func setPinned(_: Bool, perpetualId _: String) throws {}

    public func getPositions(walletId _: WalletId, address _: String) async throws {}
}

// MARK: - HyperliquidPerpetualServiceable

extension PerpetualServiceMock: HyperliquidPerpetualServiceable {
    public func getHypercorePositions(walletId _: WalletId) throws -> [GemPerpetualPosition] {
        []
    }

    public func updateBalance(walletId _: WalletId, balance _: GemPerpetualBalance) throws {}

    public func diffPositions(deleteIds _: [String], positions _: [GemPerpetualPosition], walletId _: WalletId) throws {}

    public func updatePrices(_: [String: Double]) throws {}

}
