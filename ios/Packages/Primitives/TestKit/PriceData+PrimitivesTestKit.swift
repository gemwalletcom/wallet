// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension PriceData {
    static func mock(
        asset: Asset = .mock(),
        price: Price? = .none,
        priceAlerts: [PriceAlert] = [],
        market: AssetMarket? = .mock(),
        links: [AssetLink] = [],
    ) -> PriceData {
        PriceData(
            asset: asset,
            price: price,
            priceAlerts: priceAlerts,
            market: market,
            links: links,
        )
    }
}
