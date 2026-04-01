// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPrice {
    func map() -> Price {
        Price(
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: Date(timeIntervalSince1970: TimeInterval(updatedAt)),
        )
    }
}

public extension Price {
    func map() -> GemPrice {
        GemPrice(
            price: price,
            priceChangePercentage24h: priceChangePercentage24h,
            updatedAt: Int64(updatedAt.timeIntervalSince1970),
        )
    }
}

public extension GemDelegation {
    func map() throws -> Delegation {
        try Delegation(
            base: base.map(),
            validator: validator.map(),
            price: price?.map(),
        )
    }
}

public extension Delegation {
    func map() -> GemDelegation {
        GemDelegation(
            base: base.map(),
            validator: validator.map(),
            price: price?.map(),
        )
    }
}
