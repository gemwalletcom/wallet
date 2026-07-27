// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.AssetFiatValue {
    func map() -> Gemstone.AssetFiatValue {
        Gemstone.AssetFiatValue(
            amount: amount,
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
        )
    }
}
