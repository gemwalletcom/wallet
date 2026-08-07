// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.GemSwapPriceImpactType {
    func map() -> Primitives.SwapPriceImpactType {
        switch self {
        case .positive: .positive
        case .low: .low
        case .medium: .medium
        case .high: .high
        }
    }
}

public extension Gemstone.GemSwapPriceImpact {
    func map() -> Primitives.SwapPriceImpact {
        Primitives.SwapPriceImpact(
            percentage: percentage,
            impactType: impactType.map(),
            isHigh: isHigh,
        )
    }
}
