// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

public extension PriceUpdate {
    static func mock(
        assetId: AssetId = .mock(),
        price: Double = 1,
        priceChangePercentage24h: Double = 0,
        rate: Double = 1,
    ) -> PriceUpdate {
        PriceUpdate(
            assetId: assetId,
            price: price * rate,
            priceUsd: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: .now,
        )
    }
}
