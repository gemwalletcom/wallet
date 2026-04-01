// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetual {
    func map() throws -> Primitives.Perpetual {
        try Perpetual(
            id: id,
            name: name,
            provider: provider.map(),
            assetId: AssetId(id: assetId),
            identifier: identifier,
            price: price,
            pricePercentChange24h: pricePercentChange24h,
            openInterest: openInterest,
            volume24h: volume24h,
            funding: funding,
            maxLeverage: maxLeverage,
            isIsolatedOnly: isIsolatedOnly,
        )
    }
}
