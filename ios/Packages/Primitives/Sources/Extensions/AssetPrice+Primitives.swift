// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public extension AssetPrice {
    static func empty(assetId: AssetId) -> AssetPrice {
        AssetPrice(
            assetId: assetId,
            price: 0,
            priceChangePercentage24h: 0,
            updatedAt: .now,
        )
    }

    func mapToPrice() -> Price {
        Price(
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: updatedAt,
        )
    }
}

public extension Price {
    func mapToAssetPrice(assetId: AssetId) -> AssetPrice {
        AssetPrice(
            assetId: assetId,
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: updatedAt,
        )
    }
}
