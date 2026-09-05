// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPerpetualServiceProtocol
import Primitives

public extension GemPerpetualServiceProtocol {
    func setPinned(_ isPinned: Bool, perpetualId: PerpetualId) async throws {
        try await setPinned(perpetualId: perpetualId.identifier, pinned: isPinned)
    }
}
