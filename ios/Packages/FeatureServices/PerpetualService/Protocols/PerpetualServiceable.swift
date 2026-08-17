// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol PerpetualServiceable: Sendable {
    var marketsUpdatedAt: Date? { get }
    func updateMarkets() async throws
    func clearMarkets() throws
    func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick]
    func portfolio(address: String) async throws -> PerpetualPortfolio
    func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) throws
    func getPositions(walletId: WalletId, address: String) async throws
}
