// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemMarketsRefreshTrigger
import enum Gemstone.GemPerpetualSocketUpdate
import protocol Gemstone.GemPerpetualServiceProtocol
import Primitives

public extension GemPerpetualServiceProtocol {
    func updateMarkets(trigger: GemMarketsRefreshTrigger) async throws {
        _ = try await syncMarketsIfNeeded(chain: Chain.hyperCore.rawValue, trigger: trigger)
    }

    func portfolio(address: String) async throws -> PerpetualPortfolio {
        try await PerpetualPortfolio(getPortfolio(chain: Chain.hyperCore.rawValue, address: address))
    }

    func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) async throws {
        try await setPinned(perpetualId: perpetualId.identifier, pinned: isPinned)
    }

    func applySocketMessage(walletId: WalletId, mode: PerpetualAccountMode, data: Data) async throws -> GemPerpetualSocketUpdate {
        try await applySocketMessage(walletId: walletId.id, mode: mode.json(), data: data)
    }
}
