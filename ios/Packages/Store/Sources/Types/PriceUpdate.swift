// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct PriceUpdate: Sendable {
    public let assetId: AssetId
    public let price: Double
    public let priceUsd: Double
    public let priceChangePercentage24h: Double
    public let updatedAt: Date

    public init(assetId: AssetId, price: Double, priceUsd: Double, priceChangePercentage24h: Double, updatedAt: Date) {
        self.assetId = assetId
        self.price = price
        self.priceUsd = priceUsd
        self.priceChangePercentage24h = priceChangePercentage24h
        self.updatedAt = updatedAt
    }
}
