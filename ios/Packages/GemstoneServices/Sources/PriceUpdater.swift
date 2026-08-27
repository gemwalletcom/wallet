// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemStreamSubscriptionService
import Primitives

public protocol PriceUpdater: Sendable {
    func addPrices(assetIds: [AssetId]) async throws
}

extension GemStreamSubscriptionService: PriceUpdater {
    public func addPrices(assetIds: [AssetId]) async throws {
        try await addPrices(assetIds: assetIds.map(\.identifier))
    }
}
