// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemPerpetualSocketUpdate
import protocol Gemstone.GemPerpetualServiceProtocol
import Primitives

public extension GemPerpetualServiceProtocol {
    func updateMarkets() async throws {
        _ = try await syncMarketsIfStale(chain: Chain.hyperCore.rawValue)
    }

    func candlesticks(symbol: String, period: ChartPeriod) async throws -> [ChartCandleStick] {
        try await getCandlesticks(chain: Chain.hyperCore.rawValue, symbol: symbol, period: period.json()).map { try ChartCandleStick($0) }
    }

    func portfolio(address: String) async throws -> PerpetualPortfolio {
        try await PerpetualPortfolio(getPortfolio(chain: Chain.hyperCore.rawValue, address: address))
    }

    func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) async throws {
        try await setPinned(perpetualId: perpetualId.identifier, pinned: isPinned)
    }

    @discardableResult
    func syncPositions(walletId: WalletId, address: String) async throws -> PerpetualAccountMode {
        try await PerpetualAccountMode(syncPositions(walletId: walletId.id, chain: Chain.hyperCore.rawValue, address: address))
    }

    func accountMode(walletId: WalletId, address: String) async throws -> PerpetualAccountMode {
        try await PerpetualAccountMode(accountMode(walletId: walletId.id, chain: Chain.hyperCore.rawValue, address: address))
    }

    func applySocketMessage(walletId: WalletId, mode: PerpetualAccountMode, data: Data) async throws -> GemPerpetualSocketUpdate {
        try await applySocketMessage(walletId: walletId.id, mode: mode.json(), data: data)
    }
}
