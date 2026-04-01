// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.SwapPriceImpactType {
    func map() -> Primitives.SwapPriceImpactType {
        switch self {
        case .positive: .positive
        case .low: .low
        case .medium: .medium
        case .high: .high
        }
    }
}
