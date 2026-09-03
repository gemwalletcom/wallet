// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemMarketsRefreshTrigger
import protocol Gemstone.GemPerpetualServiceProtocol
import Primitives

public extension GemPerpetualServiceProtocol {
    func updateMarkets(trigger: GemMarketsRefreshTrigger) async throws {
        _ = try await syncMarketsIfNeeded(chain: Chain.hyperCore.rawValue, trigger: trigger)
    }

    func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) async throws {
        try await setPinned(perpetualId: perpetualId.identifier, pinned: isPinned)
    }
}
